use std::collections::HashMap;
use std::net::SocketAddr;

use warp::Filter;

use crate::config::Config;

pub struct Api {
    configs: Config,
}

impl Api {
    pub fn new(configs: Config) -> Api {
        Api { configs }
    }

    pub async fn start_api(&self) {
        let addr_str = format!("{}:{}", self.configs.clone().host.unwrap(), self.configs.clone().port.unwrap());
        let addr: SocketAddr = addr_str.parse().unwrap();
        println!("Server started in host: {}", addr.to_string());
        warp::serve(Api::routes(self)).run(addr).await;
    }

    pub fn routes(&self) -> impl Filter<Extract=impl warp::Reply, Error=warp::Rejection> + Clone {
        Api::post_notification(self)
    }

    pub fn post_notification(&self) -> impl Filter<Extract=impl warp::Reply, Error=warp::Rejection> + Clone {
        let auth_token = self.configs.clone().api_token.unwrap();
        warp::path!("notification")
            .and(warp::post())
            .and(warp::body::json())
            .and(warp::header::<String>("Authentication"))
            .map(move |body: HashMap<String, String>, token: String| {
                if !auth_token.eq(&token) {
                    return "404".to_string();
                }
                // validate message to then notify to telegram
                format!("call: {}, token: {}", body.get("message").unwrap(), token)
            })
    }
}

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::sync::Mutex;
use warp::Filter;

use crate::config::Config;
use crate::monitor::telegram::TelegramService;

pub struct ApiService {
    configs: Config,
    telegram: Arc<Mutex<TelegramService>>,
}

impl ApiService {
    pub fn new(configs: Config, telegram: Arc<Mutex<TelegramService>>) -> ApiService {
        ApiService { configs, telegram }
    }

    pub async fn start_api(&self) {
        let addr_str = format!("{}:{}", self.configs.clone().host.unwrap(), self.configs.clone().port.unwrap());
        let addr: SocketAddr = addr_str.parse().unwrap();
        println!("Server started in host: {}", addr.to_string());
        warp::serve(self.routes()).run(addr).await;
    }

    pub fn routes(&self) -> impl Filter<Extract=impl warp::Reply, Error=warp::Rejection> + Clone {
        self.post_notification()
    }

    pub fn post_notification(&self) -> impl Filter<Extract=impl warp::Reply, Error=warp::Rejection> + Clone {
        let auth_token = self.configs.clone().api_token.unwrap();
        let telegram_ref = self.telegram.clone();
        warp::path!("notification")
            .and(warp::post())
            .and(warp::body::json())
            .and(warp::header::<String>("Authentication"))
            // inject auth token
            .and(warp::any().map(move || auth_token.clone()))
            // inject telegram service reference
            .and(warp::any().map(move || telegram_ref.clone()))
            .then(|body: HashMap<String, String>, token: String, auth_token: String, telegram_ref: Arc<Mutex<TelegramService>>| async move {
                // validate access token
                if !auth_token.eq(&token) {
                    return warp::reply::with_status("FORBIDDEN", warp::http::StatusCode::FORBIDDEN);
                }

                // validate message to then notify to telegram
                println!("Notification request: {:?}", body);

                telegram_ref.lock().await.send_message(body.get("message").unwrap().to_string(), &None).await;

                // 200 response
                warp::reply::with_status("ACCEPTED", warp::http::StatusCode::ACCEPTED)
            })
    }
}

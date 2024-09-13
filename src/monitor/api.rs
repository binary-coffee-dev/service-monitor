use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use reqwest::header::AUTHORIZATION;
use warp::Filter;
use tokio::sync::Mutex;
use tokio::sync::oneshot::Receiver;

use crate::config::Config;
use crate::monitor::telegram::{TelegramServiceTrait};
use crate::monitor::utils::ToMarkdown;

pub struct ApiService {
    pub configs: Config,
    pub telegram: Arc<Mutex<dyn TelegramServiceTrait + Send>>,
}

impl ApiService {
    pub fn new(configs: Config, telegram: Arc<Mutex<dyn TelegramServiceTrait + Send>>) -> ApiService {
        ApiService { configs, telegram }
    }

    pub async fn start_api(&self, kill_receiver: Option<Receiver<()>>) {
        let addr_str = format!("{}:{}", self.configs.clone().host.unwrap(), self.configs.clone().port.unwrap());
        let addr: SocketAddr = addr_str.parse().unwrap();
        println!("Server started in host: {}", addr.to_string());

        match kill_receiver {
            None => {
                warp::serve(self.routes()).run(addr).await;
            }
            Some(rx) => {
                let (_addr, server) = warp::serve(self.routes())
                    .bind_with_graceful_shutdown(addr, async {
                        rx.await.ok();
                    });
                server.await;
            }
        };
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
            .and(warp::header::<String>(AUTHORIZATION.as_str()))
            // inject auth token
            .and(warp::any().map(move || auth_token.clone()))
            // inject telegram service reference
            .and(warp::any().map(move || telegram_ref.clone()))
            .then(|body: HashMap<String, String>, token: String, auth_token: String, telegram_ref: Arc<Mutex<dyn TelegramServiceTrait + Send>>| async move {
                // validate access token
                if !ApiService::validate_auth(&auth_token, &token) {
                    return warp::reply::with_status("FORBIDDEN", warp::http::StatusCode::FORBIDDEN);
                }

                // validate message to then notify to telegram
                println!("Notification request: {:?}", body);

                // send message to telegram
                telegram_ref.lock().await.send_message(
                    body.get("message").unwrap().to_string().parse_text_to_markdown(), &None,
                ).await;

                // 200 response
                warp::reply::with_status("ACCEPTED", warp::http::StatusCode::ACCEPTED)
            })
    }

    fn validate_auth(api_token: &str, base64_token: &str) -> bool {
        let base64_token = base64_token.trim();

        let e: Option<usize> = base64_token.find(" ");
        if e.is_none() {
            return false;
        }

        if &base64_token[0..e.unwrap()] != "Basic" {
            return false;
        }

        match BASE64_STANDARD.decode(&base64_token[e.unwrap()..].trim()) {
            Ok(token) => {
                api_token.eq(&String::from_utf8(token).unwrap())
            }
            Err(_) => {
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::monitor::api::ApiService;

    #[test]
    fn validate_auth_token_test() {
        // valid tokens
        assert_eq!(true, ApiService::validate_auth("test", "Basic dGVzdA=="));
        assert_eq!(true, ApiService::validate_auth("test", " Basic dGVzdA=="));
        assert_eq!(true, ApiService::validate_auth("test", " Basic  dGVzdA==  "));

        // invalid tokens
        assert_eq!(false, ApiService::validate_auth("test", "dGVzdA==  "));
        assert_eq!(false, ApiService::validate_auth("test", "Basi cdGVzdA=="));
        assert_eq!(false, ApiService::validate_auth("tests", "Basic dGVzdA=="));

        // invalid base64
        assert_eq!(false, ApiService::validate_auth("tests", "Basic cdGVzdA=="));
    }
}

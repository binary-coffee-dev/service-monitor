use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use reqwest::header::AUTHORIZATION;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::oneshot::Receiver;
use tokio::sync::Mutex;
use warp::Filter;

use crate::config::Config;
use crate::monitor::utils::ToMarkdown;
use crate::validator::Validator;

pub struct ApiService {
    pub configs: Config,
    pub validator: Arc<Mutex<Validator>>,
}

impl ApiService {
    pub fn new(configs: Config, validator: Arc<Mutex<Validator>>) -> ApiService {
        ApiService { configs, validator }
    }

    pub async fn start_api(&self, kill_receiver: Option<Receiver<()>>) {
        let addr_str = format!(
            "{}:{}",
            self.configs.clone().host.unwrap(),
            self.configs.clone().port.unwrap()
        );
        let addr: SocketAddr = addr_str.parse().unwrap();
        println!("Server started in host: {}", addr.to_string());

        match kill_receiver {
            None => {
                warp::serve(self.routes()).run(addr).await;
            }
            Some(rx) => {
                let (_addr, server) =
                    warp::serve(self.routes()).bind_with_graceful_shutdown(addr, async {
                        rx.await.ok();
                    });
                server.await;
            }
        };
    }

    pub fn routes(
        &self,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        self.post_notification()
    }

    pub fn post_notification(
        &self,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let auth_token = self.configs.clone().api_token.unwrap();
        let validator_ref = self.validator.clone();

        warp::path!("notification")
            .and(warp::post())
            .and(warp::body::json())
            .and(warp::header::<String>(AUTHORIZATION.as_str()))
            // inject auth token
            .and(warp::any().map(move || auth_token.clone()))
            // inject telegram service reference
            .and(warp::any().map(move || validator_ref.clone()))
            .then(
                |body: HashMap<String, String>,
                 token: String,
                 auth_token: String,
                 validator: Arc<Mutex<Validator>>| async move {
                    // validate access token
                    if !ApiService::validate_auth(&auth_token, &token) {
                        return warp::reply::with_status(
                            "FORBIDDEN",
                            warp::http::StatusCode::FORBIDDEN,
                        );
                    }

                    // validate message to then notify to telegram
                    println!("Notification request: {:?}", body);

                    // send message to telegram
                    validator
                        .lock()
                        .await
                        .send_telegram_message(
                            body.get("message")
                                .unwrap()
                                .to_string()
                                .parse_text_to_markdown(),
                            &None,
                        )
                        .await;

                    // 200 response
                    warp::reply::with_status("ACCEPTED", warp::http::StatusCode::ACCEPTED)
                },
            )
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
            Ok(token) => api_token.eq(&String::from_utf8(token).unwrap()),
            Err(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::monitor::api::ApiService;
    use base64::prelude::BASE64_STANDARD;
    use base64::Engine;

    #[test]
    fn validate_base64_test() {
        assert_eq!(BASE64_STANDARD.encode("test"), "dGVzdA==");
    }

    #[test]
    fn validate_auth_token_test() {
        // valid tokens
        assert_eq!(true, ApiService::validate_auth("test", "Basic dGVzdA=="));
        assert_eq!(true, ApiService::validate_auth("test", " Basic dGVzdA=="));
        assert_eq!(
            true,
            ApiService::validate_auth("test", " Basic  dGVzdA==  ")
        );

        // invalid tokens
        assert_eq!(false, ApiService::validate_auth("test", "dGVzdA==  "));
        assert_eq!(false, ApiService::validate_auth("test", "Basi cdGVzdA=="));
        assert_eq!(false, ApiService::validate_auth("tests", "Basic dGVzdA=="));

        // invalid base64
        assert_eq!(false, ApiService::validate_auth("tests", "Basic cdGVzdA=="));
    }
}

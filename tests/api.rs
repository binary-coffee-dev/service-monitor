use std::collections::HashMap;
use std::sync::Arc;
use mockall::predicate::eq;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use reqwest::StatusCode;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;
use tokio::sync::oneshot::Sender;
use tokio::task::JoinHandle;

use sm::monitor::api::ApiService;
use sm::config::Config;
use sm::monitor::telegram::{MockTelegramServiceTrait, TelegramServiceTrait};

fn get_default_test_config(port: Option<u32>) -> Config {
    Config {
        // service monitor
        enable_service_monitor: Some(false),
        api_tests: Some(Vec::new()),
        frontend_tests: Some(Vec::new()),
        website_monitor_timeout: Some(20),
        ssl_tests: Some(Vec::new()),
        pause_reminder_timeout: Some(86400),
        times_to_retry: Some(5),
        // telegram
        enable_telegram: Some(false),
        telegram_bot_token: None,
        groups: Some(Vec::new()),
        // api
        enable_api: Some(true),
        host: Some("127.0.0.1".to_string()),
        port,
        api_token: Some("test".to_string()),
    }
}

fn start_api_service(config: Config, telegram_service: Arc<Mutex<dyn TelegramServiceTrait + Send>>) -> (JoinHandle<()>, Runtime, Sender<()>) {
    let rt = Runtime::new().unwrap();

    let (tx, rx) = tokio::sync::oneshot::channel();
    let api_thread = rt.spawn(async move {
        let api_service = ApiService::new(config, telegram_service);
        api_service.start_api(Some(rx)).await;
        println!("API service finished");
    });

    (api_thread, rt, tx)
}

fn get_url(config: Config) -> String {
    let host = config.host.clone().unwrap();
    let port = config.port.clone().unwrap();
    format!("http://{}:{}/notification", host, port)
}

#[tokio::test]
async fn test_send_notification_flow() {
    // start api service
    let config_ref = get_default_test_config(Some(8353));
    let expected_message = "test message sent to telegram".to_string();

    let telegram_service_share = Arc::new(Mutex::new(MockTelegramServiceTrait::new()));

    // start api service
    let telegram_service_ref = telegram_service_share.clone();
    let (api_thread, rt, tx) = start_api_service(config_ref.clone(), telegram_service_ref.clone());

    // assert that telegram service send_message method was called
    telegram_service_share.lock().await.expect_send_message()
        .with(eq(expected_message.clone()), eq(None))
        .times(1)
        .return_once(|_, _| {});

    // call notification endpoint
    let body: HashMap<String, String> = [("message".to_string(), expected_message.clone())].iter().cloned().collect();
    let response = reqwest::Client::new()
        .post(get_url(config_ref.clone()))
        .header(CONTENT_TYPE, "application/json")
        .header(AUTHORIZATION, "Basic dGVzdA==")
        .json(&body)
        .send()
        .await
        .expect("Failed to send notification");
    println!("Response: {:?}", response);
    assert_eq!(response.status(), StatusCode::ACCEPTED, "Status code is not 202");

    // stop api service
    println!("Sending kill signal to api service");
    if let Err(_) = tx.send(()) {
        panic!("Failed to send kill signal to api service");
    }
    api_thread.await.expect("Failed to join api thread");
    rt.shutdown_background();
}

#[tokio::test]
async fn test_authorization_api_fail() {
    // start api service
    let config_ref = get_default_test_config(Some(8354));
    let expected_message = "test message sent to telegram".to_string();

    // start api service
    let (api_thread, rt, tx) =
        start_api_service(config_ref.clone(), Arc::new(Mutex::new(MockTelegramServiceTrait::new())));

    // call notification endpoint
    let body: HashMap<String, String> = [("message".to_string(), expected_message.clone())].iter().cloned().collect();
    let response = reqwest::Client::new()
        .post(get_url(config_ref.clone()))
        .header(CONTENT_TYPE, "application/json")
        .header(AUTHORIZATION, "Basic bad_token")
        .json(&body)
        .send()
        .await
        .expect("Failed to send notification");
    println!("Response: {:?}", response);
    assert_eq!(response.status(), StatusCode::FORBIDDEN, "Status code is not 403");

    // stop api service
    println!("Sending kill signal to api service");
    if let Err(_) = tx.send(()) {
        panic!("Failed to send kill signal to api service");
    }
    api_thread.await.expect("Failed to join api thread");
    rt.shutdown_background();
}

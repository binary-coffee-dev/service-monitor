use std::collections::HashMap;
use std::sync::Arc;
use mockall::predicate::eq;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use reqwest::StatusCode;
use tokio::sync::Mutex;
use sm::monitor::api::ApiService;
use sm::config::Config;
use sm::monitor::telegram::{MockTelegramServiceTrait};

// static BEFORE_ALL: Once = Once::new();
//
// fn before_all() {
//     BEFORE_ALL.call_once(|| {
//         // setup
//     });
// }

fn get_default_test_config() -> Config {
    Config {
        telegram_bot_token: None,
        groups: Some(Vec::new()),
        api_tests: Some(Vec::new()),
        frontend_tests: Some(Vec::new()),
        website_monitor_timeout: Some(20),
        ssl_tests: Some(Vec::new()),
        pause_reminder_timeout: Some(86400),
        times_to_retry: Some(5),
        host: Some("127.0.0.1".to_string()),
        port: Some(8353),
        api_token: Some("test".to_string()),
        enable_api: Some(true),
    }
}

#[tokio::test]
async fn test_send_notification_flow() {
    // start api service
    let rt = tokio::runtime::Runtime::new().unwrap();
    let config_ref = get_default_test_config();
    let host = config_ref.host.clone().unwrap();
    let port = config_ref.port.clone().unwrap();
    let expected_message = "test message sent to telegram".to_string();

    let telegram_service_share = Arc::new(Mutex::new(MockTelegramServiceTrait::new()));
    let (tx, rx) = tokio::sync::oneshot::channel();

    // start api service
    let telegram_service_ref = telegram_service_share.clone();
    let api_thread = rt.spawn(async move {
        let api_service = ApiService::new(config_ref, telegram_service_ref);
        api_service.start_api(Some(rx)).await;
        println!("API service finished");
    });

    // assert that telegram service send_message method was called
    telegram_service_share.lock().await.expect_send_message()
        .with(eq(expected_message.clone()), eq(None))
        .times(1)
        .return_once(|_, _| {});

    // call notification endpoint
    let url = format!("http://{}:{}/notification", host, port);
    let body: HashMap<String, String> = [("message".to_string(), expected_message.clone())].iter().cloned().collect();
    let response = reqwest::Client::new()
        .post(url)
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

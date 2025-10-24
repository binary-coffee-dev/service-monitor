use mockall::predicate::eq;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use reqwest::StatusCode;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::oneshot::Sender;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use sm::monitor::api_server::ApiServer;
use sm::monitor::services::config_service::ConfigService;
use sm::monitor::services::telegram_service::{MockTelegramServiceTrait, TelegramServiceTrait};
use sm::monitor::services::website_vitality_service::WebsiteVitalityService;
use sm::monitor::validator::Validator;

fn get_default_test_config(port: Option<u32>) -> ConfigService {
    ConfigService {
        // service monitor
        enable_monitoring_service: Some(false),
        api_tests: Some(Vec::new()),
        frontend_tests: Some(Vec::new()),
        website_monitoring_interval: Some(20),
        ssl_tests: Some(Vec::new()),
        pause_reminder_interval: Some(86400),
        times_to_retry_after_error: Some(5),
        // telegram_service
        enable_telegram_bot_commands: Some(false),
        retrieve_commands_interval: Some(2),
        telegram_bot_token: None,
        groups: Some(Vec::new()),
        // api
        enable_api: Some(true),
        api_host: Some("127.0.0.1".to_string()),
        api_port: port,
        api_token: Some("test".to_string()),
    }
}

fn start_api_service(
    config: ConfigService,
    telegram_service: Arc<Mutex<dyn TelegramServiceTrait + Send>>,
) -> (JoinHandle<()>, Runtime, Sender<()>) {
    let rt = Runtime::new().unwrap();

    let (tx, rx) = tokio::sync::oneshot::channel();
    let api_thread = rt.spawn(async move {
        let validator = Arc::new(Mutex::new(Validator::new(
            telegram_service.clone(),
            Arc::new(Mutex::new(WebsiteVitalityService::new(
                config.clone(),
            ))),
        )));
        let api_service = ApiServer::new(config, validator);
        api_service.start_api(Some(rx)).await;
        println!("API service finished");
    });

    (api_thread, rt, tx)
}

fn get_url(config: ConfigService) -> String {
    let host = config.api_host.clone().unwrap();
    let port = config.api_port.clone().unwrap();
    format!("http://{}:{}/notification", host, port)
}

#[tokio::test]
async fn test_send_notification_flow() {
    // start api service
    let config_ref = get_default_test_config(Some(8353));
    let expected_message = "test message sent to telegram service".to_string();

    let telegram_service_share = Arc::new(Mutex::new(MockTelegramServiceTrait::new()));

    // start api service
    let telegram_service_ref = telegram_service_share.clone();
    let (api_thread, rt, tx) = start_api_service(config_ref.clone(), telegram_service_ref.clone());

    // assert that telegram_service service send_message method was called
    telegram_service_share
        .lock()
        .await
        .expect_send_message()
        .with(eq(expected_message.clone()), eq(None))
        .times(1)
        .return_once(|_, _| {});

    // call notification endpoint
    let body: HashMap<String, String> = [("message".to_string(), expected_message.clone())]
        .iter()
        .cloned()
        .collect();
    let response = reqwest::Client::new()
        .post(get_url(config_ref.clone()))
        .header(CONTENT_TYPE, "application/json")
        .header(AUTHORIZATION, "Basic dGVzdA==")
        .json(&body)
        .send()
        .await
        .expect("Failed to send notification");
    println!("Response: {:?}", response);
    assert_eq!(
        response.status(),
        StatusCode::ACCEPTED,
        "Status code is not 202"
    );

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
    let expected_message = "test message sent to telegram service".to_string();

    // start api service
    let (api_thread, rt, tx) = start_api_service(
        config_ref.clone(),
        Arc::new(Mutex::new(MockTelegramServiceTrait::new())),
    );

    // call notification endpoint
    let body: HashMap<String, String> = [("message".to_string(), expected_message.clone())]
        .iter()
        .cloned()
        .collect();
    let response = reqwest::Client::new()
        .post(get_url(config_ref.clone()))
        .header(CONTENT_TYPE, "application/json")
        .header(AUTHORIZATION, "Basic bad_token")
        .json(&body)
        .send()
        .await
        .expect("Failed to send notification");
    println!("Response: {:?}", response);
    assert_eq!(
        response.status(),
        StatusCode::FORBIDDEN,
        "Status code is not 403"
    );

    // stop api service
    println!("Sending kill signal to api service");
    if let Err(_) = tx.send(()) {
        panic!("Failed to send kill signal to api service");
    }
    api_thread.await.expect("Failed to join api thread");
    rt.shutdown_background();
}

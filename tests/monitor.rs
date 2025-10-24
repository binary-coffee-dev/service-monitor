use std::sync::Arc;
use httpmock::Method::GET;
use httpmock::MockServer;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;

use sm::monitor::Monitor;
use sm::monitor::services::config_service::ConfigService;
use sm::monitor::services::telegram_service::MockTelegramServiceTrait;
use sm::monitor::services::website_vitality_service::{Get, RouteTest};

fn get_default_test_config(port: Option<u32>) -> ConfigService {
    ConfigService {
        // service monitor
        enable_monitoring_service: Some(true),
        api_tests: Some(Vec::new()),
        frontend_tests: Some(Vec::new()),
        website_monitoring_interval: Some(0),
        ssl_tests: Some(Vec::new()),
        pause_reminder_interval: Some(86400),
        times_to_retry_after_error: Some(5),
        // telegram_service
        enable_telegram_bot_commands: Some(false),
        retrieve_commands_interval: Some(2),
        telegram_bot_token: None,
        groups: Some(Vec::new()),
        // api
        enable_api: Some(false),
        api_host: Some("127.0.0.1".to_string()),
        api_port: port,
        api_token: Some("test".to_string()),
    }
}

fn wait_action<F>(validate: F) where F: Fn() -> bool {
    loop {
        if validate() {
            break;
        }
    }
}

#[tokio::test]
async fn test_authorization_api_fail() {
    let rt = Runtime::new().unwrap();

    // start mock server
    let mock_server = MockServer::start();
    let mock_endpoint = mock_server.mock(|when, then| {
        when.method(GET)
            .path("/mock_endpoint");
        then.status(200)
            .header("content-type", "text/html")
            .body("response body");
    });

    // start api service
    let mut config_ref = get_default_test_config(Some(8354));
    config_ref.api_tests = Some(vec![
        RouteTest::GET(Get {
            url: mock_server.url("/mock_endpoint")
        })
    ]);

    let telegram_service_share = Arc::new(Mutex::new(MockTelegramServiceTrait::new()));

    // mock services
    let monitor = Monitor::new(config_ref.clone(), Some(telegram_service_share));
    // let monitor_thread = monitor.start();
    rt.spawn(async move {
        monitor.start().await;
        println!("API service finished");
    });

    wait_action(|| {
        return mock_endpoint.hits() > 1;
    });

    rt.shutdown_background();
}

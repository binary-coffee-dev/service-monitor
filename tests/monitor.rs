use std::sync::Arc;

use httpmock::Method::GET;
use httpmock::MockServer;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;

use sm::config::Config;
use sm::monitor::Monitor;
use sm::monitor::telegram::MockTelegramServiceTrait;
use sm::monitor::website::{Get, RouteTest};

fn get_default_test_config(port: Option<u32>) -> Config {
    Config {
        // service monitor
        enable_service_monitor: Some(true),
        api_tests: Some(Vec::new()),
        frontend_tests: Some(Vec::new()),
        website_monitor_timeout: Some(0),
        ssl_tests: Some(Vec::new()),
        pause_reminder_timeout: Some(86400),
        times_to_retry: Some(5),
        // telegram
        enable_telegram: Some(false),
        telegram_bot_token: None,
        groups: Some(Vec::new()),
        // api
        enable_api: Some(false),
        host: Some("127.0.0.1".to_string()),
        port,
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

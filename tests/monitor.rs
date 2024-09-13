use sm::config::Config;
use sm::monitor::Monitor;
use sm::monitor::website::{Get, RouteTest};
use sm::monitor::website::RouteTest::POST;

fn get_default_test_config(port: Option<u32>) -> Config {
    Config {
        // service monitor
        enable_service_monitor: Some(true),
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
        enable_api: Some(false),
        host: Some("127.0.0.1".to_string()),
        port,
        api_token: Some("test".to_string()),
    }
}

#[tokio::test]
async fn test_authorization_api_fail() {
    // start api service
    let mut config_ref = get_default_test_config(Some(8354));
    config_ref.api_tests = Some(vec![
        RouteTest::GET(Get {
            url: ""
        })
    ]);

    // mock services
    let monitor = Monitor::new(config_ref.clone());
    monitor.start().await;
}

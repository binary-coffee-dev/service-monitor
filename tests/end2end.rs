use sm::monitor::services::website_vitality_service::{Get, RouteTest};
use sm::monitor::Monitor;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::sync::Mutex;

const TEST_TIMEOUT_SECS: i64 = 60;

fn get_now_timestamp() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_secs() as i64
}

#[tokio::test]
async fn test_services_monitoring() {
    let test_start_time = get_now_timestamp();

    // Initialize monitor with configurations
    let mut configs = sm::monitor::services::config_service::ConfigService::read_configurations();
    configs.enable_telegram_bot_commands = Some(false);
    configs.retrieve_commands_interval = Some(0);
    configs.telegram_bot_token = Some("TELEGRAM_BOT_TOKEN".to_string());

    configs.enable_monitoring_service = Some(true);
    configs.website_monitoring_interval = Some(0);

    configs.enable_api = Some(false);

    // mock services to monitoring
    let mut mock_services = mockito::Server::new_async().await;
    let service1 = mock_services
        .mock("GET", "/service1")
        .with_status(200)
        .with_body_from_request(|_request| "Service 1 is up".into())
        .expect_at_least(1)
        .create_async()
        .await;
    let service2 = mock_services
        .mock("POST", "/service2")
        .with_status(200)
        .with_body_from_request(|_request| "Service 2 is up".into())
        .expect_at_least(1)
        .create_async()
        .await;
    let service3 = mock_services
        .mock("GET", "/service3")
        .with_status(200)
        .with_body_from_request(|_request| "Service 3 is up".into())
        .expect_at_least(1)
        .create_async()
        .await;
    configs.frontend_tests = Some(vec![RouteTest::GET(Get {
        url: format!("{}/service1", &mock_services.url()),
    })]);
    configs.api_tests = Some(vec![
        RouteTest::GET(Get {
            url: format!("{}/service3", &mock_services.url()),
        }),
        RouteTest::POST(sm::monitor::services::website_vitality_service::Post {
            url: format!("{}/service2", &mock_services.url()),
            body: "{}".to_string(),
            content_type: "application/json".to_string(),
        }),
    ]);

    // todo: pending tests for ssl_tests

    // start the monitor
    let monitor = Monitor::new(configs);
    let rt = tokio::runtime::Runtime::new().unwrap();
    let running_flag = Arc::new(Mutex::new(AtomicBool::new(true)));
    let running_flag_clone = running_flag.clone();
    let monitor_thread = rt.spawn(async move {
        monitor.start(Some(running_flag_clone)).await;
    });

    // expect the services to be called
    while service1.matched() && service2.matched() && service3.matched() {
        if get_now_timestamp() - test_start_time > TEST_TIMEOUT_SECS {
            panic!("Timeout waiting for list files calls");
        }
    }

    // stop the monitor
    running_flag
        .lock()
        .await
        .store(false, std::sync::atomic::Ordering::SeqCst);
    let _ = tokio::join!(monitor_thread);
    rt.shutdown_background();
}

#[tokio::test]
async fn test_telegram_bot_commands() {

}
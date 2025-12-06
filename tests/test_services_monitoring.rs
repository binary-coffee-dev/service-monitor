use crate::utils::{
    get_default_configs, get_now_timestamp, mock_services_api, start_monitor, TEST_TIMEOUT_SECS,
};
use sm::monitor::services::website_vitality_service::{Get, RouteTest};

mod utils;

#[tokio::test]
async fn test_services_monitoring() {
    let test_start_time = get_now_timestamp();

    // Initialize monitor with configurations
    let mut configs = get_default_configs();
    configs.enable_monitoring_service = Some(true);

    // mock services to monitoring
    let mut mock_services = mockito::Server::new_async().await;
    let (service1, service2, service3) = mock_services_api(&mut mock_services).await;
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
    let (monitor_thread, running_flag, rt) = start_monitor(configs);

    // expect the services to be called
    while !service1.matched() || !service2.matched() || !service3.matched() {
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

use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use crate::utils::{
    get_default_configs, get_now_timestamp, mock_services_api, mock_telegram_api, start_monitor,
    TEST_TIMEOUT_SECS,
};
use sm::monitor::services::telegram_service::models::{Chat, GetUpdatesRes, Message, MessageEntity, Update};
use sm::monitor::services::website_vitality_service::{Get, RouteTest};

mod utils;

#[tokio::test]
async fn test_check_api_command() {
    let test_start_time = get_now_timestamp();

    // Initialize monitor and telegram bot with configurations
    let mut configs = get_default_configs();
    configs.enable_telegram_bot_commands = Some(true);

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

    // mock telegram bot
    let mut mock_telegram = mockito::Server::new_async().await;
    let flag = Arc::new(AtomicBool::new(true));
    let flag_clone = flag.clone();
    let (set_commands, get_commands, get_updates, _) = mock_telegram_api(
        &mut mock_telegram,
        configs.telegram_bot_token.clone().unwrap(),
        move |_params| {
            let mut updates = GetUpdatesRes { result: vec![] };
            if flag.load(std::sync::atomic::Ordering::SeqCst) {
                flag.store(false, std::sync::atomic::Ordering::SeqCst);
                updates = GetUpdatesRes {
                    result: vec![Update {
                        update_id: 10000,
                        message: Some(Message {
                            entities: Some(vec![
                                MessageEntity {
                                    type_value: "bot_command".to_string(),
                                    offset: 0,
                                    length: 10,
                                }
                            ]),
                            chat: Chat { id: 143 },
                            text: Some("/check_all".to_string()),
                        }),
                    }],
                };
            }
            let res_body = serde_json::to_string(&updates).unwrap();
            res_body
        },
    )
    .await;

    // start the monitor
    configs.telegram_api_url = Some(mock_telegram.url());
    let (monitor_thread, running_flag, rt) = start_monitor(configs);

    // wait for commands to be synchronized
    while !set_commands.matched() || !get_commands.matched() || !get_updates.matched() {
        if get_now_timestamp() - test_start_time > TEST_TIMEOUT_SECS {
            panic!("Timeout waiting for telegram setMyCommands call");
        }
    }

    // expect the services to be called
    while !service1.matched() || !service2.matched() || !service3.matched() {
        if get_now_timestamp() - test_start_time > TEST_TIMEOUT_SECS {
            panic!("Timeout waiting for list files calls");
        }
    }

    // reset the services mocks and trigger another check_all command
    mock_services.reset();
    let (service1, service2, service3) = mock_services_api(&mut mock_services).await;
    flag_clone.store(true, std::sync::atomic::Ordering::SeqCst);
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

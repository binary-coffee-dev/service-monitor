use crate::utils::{
    get_default_configs, get_now_timestamp, mock_telegram_api, start_monitor, TEST_TIMEOUT_SECS,
};
use sm::monitor::services::telegram_service::models::GetUpdatesRes;

mod utils;

#[tokio::test]
async fn test_telegram_bot_commands_sync() {
    let test_start_time = get_now_timestamp();

    // Initialize monitor with configurations
    let mut configs = get_default_configs();
    configs.enable_telegram_bot_commands = Some(true);

    // mock telegram bot
    let mut mock_telegram = mockito::Server::new_async().await;
    let (set_commands, get_commands, _, _) = mock_telegram_api(
        &mut mock_telegram,
        configs.telegram_bot_token.clone().unwrap(),
        |_params| {
            let updates = GetUpdatesRes { result: vec![] };
            let res_body = serde_json::to_string(&updates).unwrap();
            res_body
        },
    )
    .await;

    // start the monitor
    configs.telegram_api_url = Some(mock_telegram.url());
    let (monitor_thread, running_flag, rt) = start_monitor(configs);

    // wait for commands to be synchronized
    while !set_commands.matched() && !get_commands.matched() {
        if get_now_timestamp() - test_start_time > TEST_TIMEOUT_SECS {
            panic!("Timeout waiting for telegram setMyCommands call");
        }
    }

    // stop the monitor
    running_flag
        .lock()
        .await
        .store(false, std::sync::atomic::Ordering::SeqCst);
    let _ = tokio::join!(monitor_thread);
    rt.shutdown_background();
    mock_telegram.reset();
}

#[tokio::test]
async fn test_telegram_bot_commands() {
    let test_start_time = get_now_timestamp();

    // Initialize monitor with configurations
    let mut configs = get_default_configs();
    configs.enable_telegram_bot_commands = Some(true);

    // mock telegram bot
    let mut mock_telegram = mockito::Server::new_async().await;
    let (set_commands, get_commands, get_updates, _) = mock_telegram_api(
        &mut mock_telegram,
        configs.telegram_bot_token.clone().unwrap(),
        |_params| {
            let updates = GetUpdatesRes { result: vec![] };
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

    // stop the monitor
    running_flag
        .lock()
        .await
        .store(false, std::sync::atomic::Ordering::SeqCst);
    let _ = tokio::join!(monitor_thread);
    rt.shutdown_background();
    mock_telegram.reset();
}

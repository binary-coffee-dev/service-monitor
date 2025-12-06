use mockito::Matcher::Regex;
use mockito::Mock;
use sm::monitor::services::config_service::ConfigService;
use sm::monitor::services::telegram_service::models::{
    BotCommand, GetMyCommandsRes, SetMyCommandsBody,
};
use sm::monitor::Monitor;
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use url::Url;

pub const TEST_TIMEOUT_SECS: i64 = 10;

pub fn get_now_timestamp() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_secs() as i64
}

pub fn start_monitor(
    configs: ConfigService,
) -> (
    JoinHandle<()>,
    Arc<Mutex<AtomicBool>>,
    tokio::runtime::Runtime,
) {
    // start the monitor
    let monitor = Monitor::new(configs);
    let rt = tokio::runtime::Runtime::new().unwrap();
    let running_flag = Arc::new(Mutex::new(AtomicBool::new(true)));
    let running_flag_clone = running_flag.clone();
    let monitor_thread = rt.spawn(async move {
        monitor.start(Some(running_flag_clone)).await;
    });

    (monitor_thread, running_flag, rt)
}

pub fn get_default_configs() -> ConfigService {
    let mut configs = ConfigService::read_configurations();
    configs.enable_telegram_bot_commands = Some(false);
    configs.retrieve_commands_interval = Some(0);
    configs.telegram_bot_token = Some("TELEGRAM_BOT_TOKEN".to_string());

    configs.enable_monitoring_service = Some(false);
    configs.website_monitoring_interval = Some(0);

    configs.enable_api = Some(false);
    configs
}

pub async fn mock_telegram_api(
    server: &mut mockito::Server,
    telegram_bot_token: String,
    get_updates_fn: impl Fn(&HashMap<String, String>) -> String + Send + Sync + 'static,
) -> (Mock, Mock, Mock, Mock) {
    let set_commands = server
        .mock(
            "POST",
            format!("/bot{}/setMyCommands", telegram_bot_token.clone()).as_str(),
        )
        .with_status(200)
        .with_body_from_request(|request| {
            let body = String::from_utf8(request.body().unwrap().clone()).unwrap();
            let commands: SetMyCommandsBody = serde_json::from_str(&body).unwrap();
            // println!("commands: {:?}", commands);

            assert_eq!(commands.commands.len(), 6);
            assert_eq!(commands.commands[0].command, "/check_all");
            assert_eq!(commands.commands[1].command, "/check_api");
            assert_eq!(commands.commands[2].command, "/check_frontend");
            assert_eq!(commands.commands[3].command, "/check_certs");
            assert_eq!(commands.commands[4].command, "/pause");
            assert_eq!(commands.commands[5].command, "/unpause");

            "{\"ok\":true,\"result\":true}".into()
        })
        .expect_at_least(1)
        .create_async()
        .await;
    let get_commands = server
        .mock(
            "GET",
            format!("/bot{}/getMyCommands", telegram_bot_token).as_str(),
        )
        .with_status(200)
        .with_body_from_request(|_| {
            let res_body = GetMyCommandsRes {
                ok: true,
                result: vec![
                    BotCommand {
                        command: "/check_all".to_string(),
                        description: "Validate".to_string(),
                    },
                    BotCommand {
                        command: "/check_api".to_string(),
                        description: "Validate".to_string(),
                    },
                    BotCommand {
                        command: "/check_frontend".to_string(),
                        description: "Validate".to_string(),
                    },
                    BotCommand {
                        command: "/check_certs".to_string(),
                        description: "Validate".to_string(),
                    },
                    BotCommand {
                        command: "/pause".to_string(),
                        description: "Pause validations.".to_string(),
                    },
                    BotCommand {
                        command: "/unpause".to_string(),
                        description: "Unpause validations.".to_string(),
                    },
                ],
            };
            serde_json::to_string(&res_body).unwrap().into()
        })
        .expect_at_least(1)
        .create_async()
        .await;

    let get_updates = server
        .mock(
            "GET",
            Regex(
                format!("/bot{}/getUpdates?.*", telegram_bot_token)
                    .as_str()
                    .to_string(),
            ),
        )
        .with_status(200)
        .with_body_from_request(move |request| {
            let url = Url::parse(format!("http://localhost{}", request.path_and_query()).as_str())
                .unwrap();
            let params: HashMap<String, String> = url.query_pairs().into_owned().collect();
            println!("getUpdates params: {:?}", params);

            get_updates_fn(&params).into()
        })
        .expect_at_least(1)
        .create_async()
        .await;
    
    let send_message = server
        .mock(
            "POST",
            format!("/bot{}/sendMessage", telegram_bot_token).as_str(),
        )
        .with_status(200)
        .with_body_from_request(|_request| {
            "{\"ok\":true,\"result\":{}}".into()
        })
        .expect_at_least(1)
        .create_async()
        .await;

    (set_commands, get_commands, get_updates, send_message)
}

pub async fn mock_services_api(mock_services: &mut mockito::Server) -> (Mock, Mock, Mock) {
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

    (service1, service2, service3)
}

use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;
use tokio::time::sleep;

use crate::config::Config;
use crate::monitor::api::ApiService;
use crate::monitor::telegram::TelegramService;
use crate::monitor::website::WebsiteService;

pub mod api;
pub mod telegram;
pub mod website;

pub struct Monitor {
    configs: Config,
    telegram_service: Arc<Mutex<TelegramService>>,
    web_service: Arc<Mutex<WebsiteService>>,
}

/// This class introduces three key services: Telegram integration for communication, website
/// monitoring for surveillance, and an API service for streamlined data access.
impl Monitor {
    pub fn new(configs: Config) -> Monitor {
        let telegram = Arc::new(Mutex::new(TelegramService::new(configs.clone())));
        let web = Arc::new(Mutex::new(WebsiteService::new(configs.clone())));
        Monitor {
            configs: configs.clone(),
            web_service: web.clone(),
            telegram_service: telegram.clone(),
        }
    }

    pub async fn start(&self) {
        let pause = Arc::new(Mutex::new(false));
        let rt = tokio::runtime::Runtime::new().unwrap();

        // start telegram command checker
        let pause_ref = pause.clone();
        let telegram_service_ref = self.telegram_service.clone();
        let web_service_ref = self.web_service.clone();
        let telegram_monitor_thread = rt.spawn(async move {
            let telegram_monitor = TelegramMonitor::new(
                telegram_service_ref,
                web_service_ref,
                pause_ref,
            );
            telegram_monitor.start_monitoring().await
        });

        // start web monitoring
        let config_ref = self.configs.clone();
        let pause_ref = pause.clone();
        let telegram_service_ref = self.telegram_service.clone();
        let web_service_ref = self.web_service.clone();
        let website_monitor = rt.spawn(async move {
            let web_monitor = WebMonitor::new(config_ref, telegram_service_ref, web_service_ref, pause_ref.clone());
            web_monitor.run_website_monitor().await;
        });

        // start api service
        let config_ref = self.configs.clone();
        let telegram_service_ref = self.telegram_service.clone();
        let api_service = rt.spawn(async move {
            if config_ref.enable_api.unwrap() {
                let api_service = ApiService::new(config_ref, telegram_service_ref);
                api_service.start_api().await
            }
        });

        let _result = tokio::join!(telegram_monitor_thread, website_monitor, api_service);
    }
}

struct WebMonitor {
    configs: Config,
    telegram: Arc<Mutex<TelegramService>>,
    web: Arc<Mutex<WebsiteService>>,
    pause_service: Arc<Mutex<bool>>,
    validator: Arc<Mutex<Validator>>,
}

impl WebMonitor {
    pub fn new(configs: Config, telegram: Arc<Mutex<TelegramService>>, web: Arc<Mutex<WebsiteService>>, pause_service: Arc<Mutex<bool>>) -> WebMonitor {
        let validator = Arc::new(Mutex::new(Validator::new(telegram.clone(), web.clone())));
        WebMonitor { configs, telegram, web, pause_service, validator }
    }

    pub async fn run_website_monitor(&self) {
        let mut pause_time_ac = 0;
        loop {
            if pause_time_ac >= self.configs.pause_reminder_timeout.unwrap() {
                self.telegram.lock().await.send_message(
                    "⚠️ REMINDER\nService monitor is in pause.".to_string(),
                    &None,
                ).await;
            }

            let pause_v = self.pause_service.lock().await;
            if !*pause_v {
                let errors = self.web.lock().await.sumary().await;

                if !errors.is_empty() {
                    for err in errors.iter() {
                        println!("Err: {}", err);
                    }

                    self.validator.lock().await.handler_validation(errors, None, None).await;
                }
            } else {
                pause_time_ac += self.configs.website_monitor_timeout.unwrap();
            }

            sleep(Duration::from_secs(self.configs.website_monitor_timeout.unwrap())).await;
        }
    }
}

struct TelegramMonitor {
    telegram: Arc<Mutex<TelegramService>>,
    pause_service: Arc<Mutex<bool>>,
    validator: Arc<Mutex<Validator>>,
}

impl TelegramMonitor {
    pub fn new(telegram: Arc<Mutex<TelegramService>>, web: Arc<Mutex<WebsiteService>>, pause_service: Arc<Mutex<bool>>) -> TelegramMonitor {
        let validator = Arc::new(Mutex::new(Validator::new(telegram.clone(), web.clone())));
        TelegramMonitor { telegram, pause_service, validator }
    }

    pub async fn start_monitoring(&self) {
        TelegramMonitor::run_commands_sync(self).await;
        TelegramMonitor::run_telegram_monitor(self).await;
    }

    async fn run_commands_sync(&self) {
        self.telegram.lock().await.sync_commands().await;
        let commands = self.telegram.lock().await.get_commands().await;
        println!("commands: {:?}", commands);
    }

    async fn run_telegram_monitor(&self) {
        loop {
            self.telegram.lock().await.send_pendings_messages().await;
            let updates = self.telegram.lock().await.get_all_updates().await;
            if !updates.is_empty() {
                println!("--------------------");
                println!("{:?}", updates);
            }
            for update in updates {
                if let Some(msg) = update.message {
                    if let Some(ent) = msg.entities {
                        let text = msg.text.unwrap();
                        let group_id = msg.chat.id;

                        for e in ent.iter() {
                            if e.type_value == String::from("bot_command") {
                                let offset_beg = e.offset as usize;
                                let offset_end = (e.offset + e.length) as usize;
                                let command_name = TelegramMonitor::extract_command(
                                    text.to_string()[offset_beg..offset_end].to_string(),
                                );

                                println!("command: {}", command_name);

                                match command_name.as_str() {
                                    "/check_all" => {
                                        self.validator.lock().await.execute_check_api(group_id).await;
                                        self.validator.lock().await.execute_check_frontend(group_id).await;
                                        self.validator.lock().await.execute_check_certs(group_id).await;
                                    }
                                    "/check_api" => {
                                        self.validator.lock().await.execute_check_api(group_id).await;
                                    }
                                    "/check_frontend" => {
                                        self.validator.lock().await.execute_check_frontend(group_id).await;
                                    }
                                    "/check_certs" => {
                                        self.validator.lock().await.execute_check_certs(group_id).await;
                                    }
                                    "/pause" => {
                                        let mut pause_v = self.pause_service.lock().await;
                                        *pause_v = true;
                                        self.telegram.lock().await.send_message("✅ Service is paused, if you want to reanudate it use the command /unpause.".to_string(), &None).await;
                                    }
                                    "/unpause" => {
                                        let mut pause_v = self.pause_service.lock().await;
                                        *pause_v = false;
                                        self.telegram.lock().await.send_message(
                                            "✅ Service is reanudated.".to_string(),
                                            &None,
                                        ).await;
                                    }
                                    _ => {
                                        println!("⚠️ Unknow command: {}", command_name);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            sleep(Duration::from_secs(2)).await;
        }
    }

    fn extract_command(command: String) -> String {
        if let Some(index) = command.find('@') {
            return command[0..index].to_string();
        }
        return command;
    }
}

struct Validator {
    telegram: Arc<Mutex<TelegramService>>,
    web: Arc<Mutex<WebsiteService>>,
}

impl Validator {
    pub fn new(telegram: Arc<Mutex<TelegramService>>, web: Arc<Mutex<WebsiteService>>) -> Validator {
        Validator { telegram, web }
    }

    async fn execute_check_certs(&self, group_id: i64) {
        let errs = self.web.lock().await.certificates_vitaly().await;
        self.handler_validation(
            errs,
            Some("✅ Certificates are OK.".to_string()),
            Some(vec![group_id]),
        ).await;
    }

    async fn execute_check_frontend(&self, group_id: i64) {
        let errs = self.web.lock().await.frontend_vitaly().await;
        self.handler_validation(
            errs,
            Some("✅ Frontend is working fine.".to_string()),
            Some(vec![group_id]),
        ).await;
    }

    pub async fn execute_check_api(&self, group_id: i64) {
        let errs = self.web.lock().await.api_vitaly().await;
        self.handler_validation(
            errs,
            Some("✅ Api is working fine.".to_string()),
            Some(vec![group_id]),
        ).await;
    }

    pub async fn handler_validation(
        &self,
        errs: Vec<String>,
        success_msg: Option<String>,
        group_ids: Option<Vec<i64>>,
    ) {
        match success_msg {
            Some(msg) => {
                self.telegram.lock().await.send_message(Validator::handler_errors(&errs, msg), &group_ids)
                    .await;
            }
            None => {
                if !errs.is_empty() {
                    self.telegram.lock().await.send_message(Validator::handler_errors(&errs, "".to_string()), &group_ids)
                        .await;
                }
            }
        }
    }

    fn handler_errors(errs: &Vec<String>, default: String) -> String {
        if errs.len() > 0 {
            let mut report = "".to_string();
            for err in errs {
                report.push_str(&err);
                report.push_str("\n");
            }
            return report;
        }
        return default;
    }
}

#[cfg(test)]
mod tests {
    use super::{TelegramMonitor};

    #[test]
    fn extract_command_test() {
        assert_eq!(
            "/check_all",
            TelegramMonitor::extract_command("/check_all@monitor_bc_bot".to_string())
        );
        assert_eq!(
            "/check_all",
            TelegramMonitor::extract_command("/check_all".to_string())
        );
    }
}

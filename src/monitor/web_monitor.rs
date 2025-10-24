use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::sleep;

use crate::monitor::services::config_service::ConfigService;
use crate::monitor::utils::ToMarkdown;
use crate::monitor::validator::Validator;

pub struct WebMonitor {
    configs: ConfigService,
    pause_service: Arc<Mutex<bool>>,
    validator: Arc<Mutex<Validator>>,
}

impl WebMonitor {
    pub fn new(configs: ConfigService, validator: Arc<Mutex<Validator>>, pause_service: Arc<Mutex<bool>>) -> WebMonitor {
        WebMonitor { configs, pause_service, validator }
    }

    pub async fn run_website_monitor(&self) {
        let mut pause_time_ac = 0;
        loop {
            if pause_time_ac >= self.configs.pause_reminder_interval.unwrap() {
                pause_time_ac = 0;
                self.validator.lock().await.send_telegram_message(
                    "⚠️ REMINDER\nService monitor is in pause.".to_string().parse_text_to_markdown(),
                    &None,
                ).await;
            }

            let pause_v = self.pause_service.lock().await;
            if !*pause_v {
                let errors = self.validator.lock().await.get_summary_website_errors().await;

                if !errors.is_empty() {
                    for err in errors.iter() {
                        println!("Err: {}", err);
                    }

                    self.validator.lock().await.handler_validation(errors, None, None).await;
                }
            } else {
                pause_time_ac += self.configs.website_monitoring_interval.unwrap();
            }

            sleep(Duration::from_secs(self.configs.website_monitoring_interval.unwrap())).await;
        }
    }
}

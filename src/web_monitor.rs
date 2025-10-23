use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;
use tokio::time::sleep;

use crate::config::Config;
use crate::monitor::telegram::TelegramServiceTrait;
use crate::monitor::website::WebsiteService;
use crate::validator::Validator;
use crate::monitor::utils::ToMarkdown;

pub struct WebMonitor {
    configs: Config,
    telegram: Arc<Mutex<dyn TelegramServiceTrait + Send>>,
    web: Arc<Mutex<WebsiteService>>,
    pause_service: Arc<Mutex<bool>>,
    validator: Arc<Mutex<Validator>>,
}

impl WebMonitor {
    pub fn new(configs: Config, telegram: Arc<Mutex<dyn TelegramServiceTrait + Send>>, web: Arc<Mutex<WebsiteService>>, pause_service: Arc<Mutex<bool>>) -> WebMonitor {
        let validator = Arc::new(Mutex::new(Validator::new(telegram.clone(), web.clone())));
        WebMonitor { configs, telegram, web, pause_service, validator }
    }

    pub async fn run_website_monitor(&self) {
        let mut pause_time_ac = 0;
        loop {
            if pause_time_ac >= self.configs.pause_reminder_timeout.unwrap() {
                pause_time_ac = 0;
                self.telegram.lock().await.send_message(
                    "⚠️ REMINDER\nService monitor is in pause.".to_string().parse_text_to_markdown(),
                    &None,
                ).await;
            }

            let pause_v = self.pause_service.lock().await;
            if !*pause_v {
                let errors = self.web.lock().await.summary().await;

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

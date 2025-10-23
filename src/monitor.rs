use std::sync::Arc;

use tokio::sync::Mutex;

use crate::config::Config;
use crate::monitor::api::ApiService;
use crate::monitor::telegram::TelegramService;
use crate::monitor::telegram::TelegramServiceTrait;
use crate::monitor::website::WebsiteService;
use crate::telegram_monitor::TelegramMonitor;
use crate::validator::Validator;
use crate::web_monitor::WebMonitor;

pub mod api;
pub mod telegram;
pub mod website;
pub mod utils;

pub struct Monitor {
    configs: Config,
    validator: Arc<Mutex<Validator>>,
}

/// This class introduces three key services: Telegram integration for communication, website
/// monitoring for surveillance, and an API service for streamlined data access.
impl Monitor {
    pub fn new(configs: Config, telegram_ins: Option<Arc<Mutex<dyn TelegramServiceTrait + Send>>>) -> Monitor {
        let web = Arc::new(Mutex::new(WebsiteService::new(configs.clone())));
        let validator;
        if telegram_ins.is_none() {
            validator = Arc::new(Mutex::new(Validator::new(
                Arc::new(Mutex::new(TelegramService::new(configs.clone()))),
                web.clone(),
            )));
        } else {
            validator = Arc::new(Mutex::new(Validator::new(
                telegram_ins.unwrap(),
                web.clone(),
            )));
        }
        Monitor {
            configs,
            validator,
        }
    }

    pub async fn start(&self) {
        let pause = Arc::new(Mutex::new(false));
        let rt = tokio::runtime::Runtime::new().unwrap();

        // start telegram command checker
        let pause_ref = pause.clone();
        let config_ref = self.configs.clone();
        let validator_ref = self.validator.clone();
        let telegram_monitor_thread = rt.spawn(async move {
            if config_ref.enable_telegram.unwrap() {
                let telegram_monitor = TelegramMonitor::new(
                    validator_ref,
                    pause_ref,
                );
                telegram_monitor.start_monitoring().await;
                println!("Telegram monitor finished");
            }
        });

        // start web monitoring
        let config_ref = self.configs.clone();
        let pause_ref = pause.clone();
        let validator_ref = self.validator.clone();
        let website_monitor = rt.spawn(async move {
            if config_ref.enable_service_monitor.unwrap() {
                let web_monitor = WebMonitor::new(config_ref, validator_ref, pause_ref);
                web_monitor.run_website_monitor().await;
            }
        });

        // start api service
        let config_ref = self.configs.clone();
        let validator_ref = self.validator.clone();
        let api_thread = rt.spawn(async move {
            if config_ref.enable_api.unwrap() {
                let api_service = ApiService::new(config_ref, validator_ref);
                api_service.start_api(None).await;
            }
        });

        let _result = tokio::join!(telegram_monitor_thread, website_monitor, api_thread);
        rt.shutdown_background();
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

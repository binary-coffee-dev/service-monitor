use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tokio::sync::Mutex;

use crate::monitor::api_server::ApiServer;
use crate::monitor::services::config_service::ConfigService;
use crate::monitor::services::telegram_service::TelegramService;
use crate::monitor::services::website_vitality_service::WebsiteVitalityService;
use crate::monitor::telegram_monitor::TelegramMonitor;
use crate::monitor::validator::Validator;
use crate::monitor::web_monitor::WebMonitor;

pub mod api_server;
pub mod services;
pub mod telegram_monitor;
pub mod utils;
pub mod validator;
pub mod web_monitor;

pub struct Monitor {
    configs: ConfigService,
    telegram_service: Arc<Mutex<TelegramService>>,
}

/// This class introduces three key services: Telegram integration for communication, website
/// monitoring for surveillance, and an API service for streamlined data access.
impl Monitor {
    pub fn new(configs: ConfigService) -> Monitor {
        Monitor {
            configs: configs.clone(),
            telegram_service: Arc::new(Mutex::new(TelegramService::new(configs))),
        }
    }

    pub async fn start(&self, running_flag: Option<Arc<Mutex<AtomicBool>>>) {
        let pause = Arc::new(Mutex::new(false));
        let rt = tokio::runtime::Runtime::new().unwrap();

        // start telegram_service command checker
        let pause_ref = pause.clone();
        let config_ref = self.configs.clone();
        let validator_ref = self.new_validator();
        let running_flag_ref = running_flag.clone();
        let telegram_monitor_thread = rt.spawn(async move {
            if config_ref.enable_telegram_bot_commands.unwrap() {
                let telegram_monitor = TelegramMonitor::new(validator_ref, pause_ref, config_ref);
                telegram_monitor.start_monitoring(running_flag_ref).await;
                println!("Telegram monitor finished");
            }
        });

        // start web monitoring
        let config_ref = self.configs.clone();
        let pause_ref = pause.clone();
        let validator_ref = self.new_validator();
        let running_flag_ref = running_flag.clone();
        let website_monitor = rt.spawn(async move {
            if config_ref.enable_monitoring_service.unwrap() {
                let web_monitor = WebMonitor::new(config_ref, validator_ref, pause_ref);
                web_monitor.run_website_monitor(running_flag_ref).await;
            }
        });

        // start api service
        let config_ref = self.configs.clone();
        let validator_ref = self.new_validator();
        let running_flag_ref = running_flag.clone();
        let api_thread = rt.spawn(async move {
            if config_ref.enable_api.unwrap() {
                let api_service = ApiServer::new(config_ref, validator_ref);
                api_service.start_api(running_flag_ref).await;
            }
        });

        let _result = tokio::join!(telegram_monitor_thread, website_monitor, api_thread);
        rt.shutdown_background();
    }

    fn new_validator(&self) -> Arc<Mutex<Validator>> {
        Arc::new(Mutex::new(Validator::new(
            self.telegram_service.clone(),
            Arc::new(Mutex::new(WebsiteVitalityService::new(
                self.configs.clone(),
            ))),
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::TelegramMonitor;

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

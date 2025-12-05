use crate::monitor::services::config_service::ConfigService;
use crate::monitor::Monitor;

pub mod monitor;

pub async fn run() {
    let configs = ConfigService::read_configurations();
    if configs.validate_configurations() {
        let monitor = Monitor::new(ConfigService::read_configurations());
        monitor.start(None).await;
    } else {
        eprintln!("Invalid configurations provided. Please check the configuration file.");
    }
}

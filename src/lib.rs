use crate::monitor::Monitor;
use crate::monitor::services::config_service::ConfigService;

pub mod monitor;

pub async fn run() {
    let monitor = Monitor::new(ConfigService::read_configurations(), None);
    monitor.start().await;
}

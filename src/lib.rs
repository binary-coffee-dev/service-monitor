use crate::config::Config;
use crate::monitor::Monitor;

pub mod config;
pub mod monitor;

pub async fn run() {
    let monitor = Monitor::new(Config::read_configurations(), None);
    monitor.start().await;
}

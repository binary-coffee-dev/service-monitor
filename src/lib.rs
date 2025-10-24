use crate::config::Config;
use crate::monitor::Monitor;

pub mod config;
pub mod monitor;
pub mod telegram_monitor;
pub mod validator;
pub mod web_monitor;
pub mod utils;
pub mod api_server;

pub async fn run() {
    let monitor = Monitor::new(Config::read_configurations(), None);
    monitor.start().await;
}

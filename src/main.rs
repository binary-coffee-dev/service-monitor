use crate::config::Config;
use crate::monitor::Monitor;

pub mod config;
pub mod monitor;

#[tokio::main]
async fn main() {
    let monitor = Monitor::new(Config::read_configurations());

    monitor.start().await;
}

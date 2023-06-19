#[macro_use]
extern crate rocket;

use std::time::Duration;
use tokio::time::sleep;

use crate::api::start_server;
use crate::telegram::Telegram;
use crate::website::Website;

pub mod api;
pub mod telegram;
pub mod website;

fn main() {
    let telegram_communication = Telegram::new();
    let mut website_validator = Website::new(telegram_communication.clone());

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.spawn(async {
        loop {
            sleep(Duration::from_secs(4)).await;
            println!("hello");
        }
    });

    start_server().unwrap();
}

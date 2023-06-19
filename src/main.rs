#[macro_use]
extern crate rocket;

use std::time::Duration;
use tokio::time::sleep;

use crate::api::start_server;
use crate::telegram::Telegram;

pub mod api;
pub mod telegram;

fn main() {
    let mut telegramCon = Telegram::new();

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.spawn(async {
        loop {
            sleep(Duration::from_secs(1)).await;
            println!("hello");
        }
    });

    start_server().unwrap();
}

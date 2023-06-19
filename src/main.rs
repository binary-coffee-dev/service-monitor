#[macro_use] extern crate rocket;

use crate::api::start_server;

pub mod api;
// pub mod telegram;

fn main() {
    start_server().unwrap();
}

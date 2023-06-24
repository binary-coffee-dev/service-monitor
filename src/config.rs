use std::env::current_dir;
use std::{fs::File, io::BufReader};

use serde::Deserialize;

use crate::monitor::website::RouteTest;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub telegram_bot_token: String,
    pub groups: Vec<i64>,
    pub api_tests: Vec<RouteTest>,
    pub frontend_tests: Vec<RouteTest>,
    pub website_monitor_timeout: u64,
}

impl Config {
    pub fn read_configurations() -> Config {
        let mut configs = Config::default();

        // todo: take this path from the application args
        let path = String::from(format!(
            "{}/config.json",
            current_dir().unwrap().display().to_string()
        ));
        println!("path: {}", path);

        match File::open(path.clone()) {
            Ok(file) => {
                let reader = BufReader::new(file);

                // Read the JSON contents of the file as an instance of `User`.
                configs = serde_json::from_reader(reader)
                    .expect("Error deserializing configuration json file.");
            }
            Err(_) => {
                println!("File '{}' couldn't be opened.", path);
            }
        };

        println!("{:?}", configs);

        return configs;
    }

    fn default() -> Config {
        Config {
            telegram_bot_token: String::from(""),
            groups: Vec::new(),
            api_tests: Vec::new(),
            frontend_tests: Vec::new(),
            website_monitor_timeout: 20,
        }
    }
}

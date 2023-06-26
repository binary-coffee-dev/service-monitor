use std::env::current_dir;
use std::{fs::File, io::BufReader};

use serde::Deserialize;

use crate::monitor::website::{Get, RouteTest};

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub telegram_bot_token: Option<String>,
    pub groups: Option<Vec<i64>>,
    pub api_tests: Option<Vec<RouteTest>>,
    pub frontend_tests: Option<Vec<RouteTest>>,
    pub website_monitor_timeout: Option<u64>,
    pub ssl_tests: Option<Vec<Get>>,
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
                configs = Config::merge_configs_with_defalt(
                    serde_json::from_reader(reader)
                        .expect("Error deserializing configuration json file."),
                );
            }
            Err(_) => {
                println!("File '{}' couldn't be opened.", path);
            }
        };

        if configs.telegram_bot_token.is_none() {
            panic!("Telegram bot token wasn't set in the configurations.");
        }
        println!("{:?}", configs);

        return configs;
    }

    fn merge_configs_with_defalt(mut config: Config) -> Config {
        let default = Config::default();
        if config.groups.is_none() {
            config.groups = default.groups;
        }
        if config.api_tests.is_none() {
            config.api_tests = default.api_tests;
        }
        if config.frontend_tests.is_none() {
            config.frontend_tests = default.frontend_tests;
        }
        if config.website_monitor_timeout.is_none() {
            config.website_monitor_timeout = default.website_monitor_timeout;
        }
        if config.ssl_tests.is_none() {
            config.ssl_tests = default.ssl_tests;
        }
        return config;
    }

    fn default() -> Config {
        Config {
            telegram_bot_token: None,
            groups: Some(Vec::new()),
            api_tests: Some(Vec::new()),
            frontend_tests: Some(Vec::new()),
            website_monitor_timeout: Some(20),
            ssl_tests: Some(Vec::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Config;

    #[test]
    fn merge_configs_test() {
        let mut config = Config {
            telegram_bot_token: Some("asdfasdf.asdfasdf".to_string()),
            groups: None,
            api_tests: None,
            frontend_tests: None,
            website_monitor_timeout: None,
            ssl_tests: None,
        };

        config = Config::merge_configs_with_defalt(config);

        assert!(config.telegram_bot_token.is_some());
        assert!(config.groups.is_some());
        assert!(config.api_tests.is_some());
        assert!(config.frontend_tests.is_some());
        assert!(config.website_monitor_timeout.is_some());
        assert!(config.ssl_tests.is_some());
    }
}

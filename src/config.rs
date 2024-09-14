use std::env::current_dir;
use std::{fs::File, io::BufReader};

use serde::Deserialize;

use crate::monitor::website::{Get, RouteTest};

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    // service monitor
    pub enable_service_monitor: Option<bool>,
    pub api_tests: Option<Vec<RouteTest>>,
    pub frontend_tests: Option<Vec<RouteTest>>,
    pub ssl_tests: Option<Vec<Get>>,
    pub website_monitor_timeout: Option<u64>,
    pub pause_reminder_timeout: Option<u64>,
    pub times_to_retry: Option<i64>,

    // telegram
    pub enable_telegram: Option<bool>,
    pub telegram_bot_token: Option<String>,
    pub groups: Option<Vec<i64>>,

    // api
    pub enable_api: Option<bool>,
    pub host: Option<String>,
    pub port: Option<u32>,
    pub api_token: Option<String>,
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
        // println!("{:?}", configs);

        return configs;
    }

    fn merge_configs_with_defalt(mut config: Config) -> Config {
        let default = Config::default();
        // service monitor
        if config.enable_service_monitor.is_none() {
            config.enable_service_monitor = default.enable_service_monitor;
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
        if config.pause_reminder_timeout.is_none() {
            config.pause_reminder_timeout = default.pause_reminder_timeout;
        }
        if config.times_to_retry.is_none() {
            config.times_to_retry = default.times_to_retry;
        }
        // telegram
        if config.enable_telegram.is_none() {
            config.enable_telegram = default.enable_telegram;
        }
        if config.groups.is_none() {
            config.groups = default.groups;
        }
        // api
        if config.host.is_none() {
            config.host = default.host;
        }
        if config.port.is_none() {
            config.port = default.port;
        }
        if config.api_token.is_none() {
            config.api_token = default.api_token;
        }
        if config.enable_api.is_none() {
            config.enable_api = default.enable_api;
        }
        return config;
    }

    fn default() -> Config {
        Config {
            // service monitor
            enable_service_monitor: Some(true),
            api_tests: Some(Vec::new()),
            frontend_tests: Some(Vec::new()),
            website_monitor_timeout: Some(20),
            ssl_tests: Some(Vec::new()),
            pause_reminder_timeout: Some(86400),
            times_to_retry: Some(5),
            // telegram
            enable_telegram: Some(true),
            telegram_bot_token: None,
            groups: Some(Vec::new()),
            // api
            host: Some("127.0.0.1".to_string()),
            port: Some(5353),
            api_token: Some("service_token".to_string()),
            enable_api: Some(true),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Config;

    #[test]
    fn deserialize_api_endpoints_test() {
        let json_example = "{\"enable_api\": true, \"host\": \"127.0.0.1\", \"port\": 6565, \"api_token\": \"example_token\", \"telegram_bot_token\": \"123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11\", \"groups\": [149770819], \"website_monitor_timeout\": 20, \"api_tests\": [{\"type\": \"POST\", \"url\": \"https://api.binarycoffee.dev/graphql\", \"body\": \"{}\", \"content_type\": \"application/json\"}, {\"type\": \"GET\", \"url\": \"https://api.binarycoffee.dev/api/sitemap\"}], \"frontend_tests\": [{\"type\": \"GET\", \"url\": \"https://binarycoffee.dev\"}, {\"type\": \"GET\", \"url\": \"https://binarycoffee.dev/post/bienvenidos-al-blog-binary-coffeermdcl\"}, {\"type\": \"GET\", \"url\": \"https://binarycoffee.dev/users/guille\"}], \"ssl_tests\": [{\"url\": \"binarycoffee.dev\"}, {\"url\": \"api.binarycoffee.dev\"}]}".to_string();
        let configs = Config::merge_configs_with_defalt(
            serde_json::from_str(&json_example)
                .expect("Error deserializing configuration json file."),
        );

        assert!(configs.api_tests.is_some());
        assert_eq!(configs.api_tests.unwrap().len(), 2);
    }

    #[test]
    fn merge_configs_test() {
        let mut config = Config {
            // service monitor
            enable_service_monitor: None,
            api_tests: None,
            frontend_tests: None,
            ssl_tests: None,
            website_monitor_timeout: None,
            pause_reminder_timeout: None,
            times_to_retry: None,
            // telegram
            enable_telegram: None,
            telegram_bot_token: Some("asdfasdf.asdfasdf".to_string()),
            groups: None,
            // api
            host: None,
            port: None,
            api_token: None,
            enable_api: None,
        };

        config = Config::merge_configs_with_defalt(config);

        // service monitor
        assert!(config.enable_service_monitor.is_some());
        assert!(config.api_tests.is_some());
        assert!(config.frontend_tests.is_some());
        assert!(config.ssl_tests.is_some());
        assert!(config.website_monitor_timeout.is_some());
        assert!(config.pause_reminder_timeout.is_some());
        assert!(config.times_to_retry.is_some());
        // telegram
        assert!(config.enable_telegram.is_some());
        assert!(config.telegram_bot_token.is_some());
        assert!(config.groups.is_some());
        // api
        assert!(config.host.is_some());
        assert!(config.port.is_some());
        assert!(config.api_token.is_some());
        assert!(config.enable_api.is_some());
    }
}

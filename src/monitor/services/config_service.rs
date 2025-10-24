use std::env::current_dir;
use std::{fs::File, io::BufReader};
use serde::Deserialize;

use crate::monitor::services::website_vitality_service::{Get, RouteTest};

#[derive(Deserialize, Debug, Clone)]
pub struct ConfigService {
    // service monitor
    pub enable_monitoring_service: Option<bool>,

    // telegram_service
    pub enable_telegram_bot_commands: Option<bool>,
    pub retrieve_commands_interval: Option<u64>,
    pub telegram_bot_token: Option<String>,
    pub groups: Option<Vec<i64>>,

    // api
    pub enable_api: Option<bool>,
    pub api_host: Option<String>,
    pub api_port: Option<u32>,
    pub api_token: Option<String>,

    // general settings can be added here
    pub times_to_retry_after_error: Option<i64>,
    pub pause_reminder_interval: Option<u64>,
    pub api_tests: Option<Vec<RouteTest>>,
    pub frontend_tests: Option<Vec<RouteTest>>,
    pub ssl_tests: Option<Vec<Get>>,
    pub website_monitoring_interval: Option<u64>,
}

impl ConfigService {
    pub fn read_configurations() -> ConfigService {
        let mut configs = ConfigService::default();

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
                configs = ConfigService::merge_configs_with_defalt(
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

    fn merge_configs_with_defalt(mut config: ConfigService) -> ConfigService {
        let default = ConfigService::default();
        // service monitor
        if config.enable_monitoring_service.is_none() {
            config.enable_monitoring_service = default.enable_monitoring_service;
        }
        if config.api_tests.is_none() {
            config.api_tests = default.api_tests;
        }
        if config.frontend_tests.is_none() {
            config.frontend_tests = default.frontend_tests;
        }
        if config.website_monitoring_interval.is_none() {
            config.website_monitoring_interval = default.website_monitoring_interval;
        }
        if config.ssl_tests.is_none() {
            config.ssl_tests = default.ssl_tests;
        }
        if config.pause_reminder_interval.is_none() {
            config.pause_reminder_interval = default.pause_reminder_interval;
        }
        if config.times_to_retry_after_error.is_none() {
            config.times_to_retry_after_error = default.times_to_retry_after_error;
        }
        // telegram_service
        if config.enable_telegram_bot_commands.is_none() {
            config.enable_telegram_bot_commands = default.enable_telegram_bot_commands;
        }
        if config.retrieve_commands_interval.is_none() {
            config.retrieve_commands_interval = default.retrieve_commands_interval;
        }
        if config.groups.is_none() {
            config.groups = default.groups;
        }
        // api
        if config.api_host.is_none() {
            config.api_host = default.api_host;
        }
        if config.api_port.is_none() {
            config.api_port = default.api_port;
        }
        if config.api_token.is_none() {
            config.api_token = default.api_token;
        }
        if config.enable_api.is_none() {
            config.enable_api = default.enable_api;
        }
        return config;
    }

    fn default() -> ConfigService {
        ConfigService {
            // service monitor
            enable_monitoring_service: Some(true),
            // telegram
            enable_telegram_bot_commands: Some(true),
            retrieve_commands_interval: Some(2),
            telegram_bot_token: None,
            groups: Some(Vec::new()),
            // api
            api_host: Some("0.0.0.0".to_string()),
            api_port: Some(5353),
            api_token: Some("service_token".to_string()),
            enable_api: Some(true),
            // general
            api_tests: Some(Vec::new()),
            frontend_tests: Some(Vec::new()),
            website_monitoring_interval: Some(20),
            ssl_tests: Some(Vec::new()),
            pause_reminder_interval: Some(86400),
            times_to_retry_after_error: Some(5),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ConfigService;

    #[test]
    fn deserialize_api_endpoints_test() {
        let json_example = "{\"enable_api\": true, \"host\": \"127.0.0.1\", \"port\": 6565, \"api_token\": \"example_token\", \"telegram_bot_token\": \"123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11\", \"groups\": [149770819], \"website_monitoring_interval\": 20, \"api_tests\": [{\"type\": \"POST\", \"url\": \"https://api.binarycoffee.dev/graphql\", \"body\": \"{}\", \"content_type\": \"application/json\"}, {\"type\": \"GET\", \"url\": \"https://api.binarycoffee.dev/api/sitemap\"}], \"frontend_tests\": [{\"type\": \"GET\", \"url\": \"https://binarycoffee.dev\"}, {\"type\": \"GET\", \"url\": \"https://binarycoffee.dev/post/bienvenidos-al-blog-binary-coffeermdcl\"}, {\"type\": \"GET\", \"url\": \"https://binarycoffee.dev/users/guille\"}], \"ssl_tests\": [{\"url\": \"binarycoffee.dev\"}, {\"url\": \"api.binarycoffee.dev\"}]}".to_string();
        let configs = ConfigService::merge_configs_with_defalt(
            serde_json::from_str(&json_example)
                .expect("Error deserializing configuration json file."),
        );

        assert!(configs.api_tests.is_some());
        assert_eq!(configs.api_tests.unwrap().len(), 2);
    }

    #[test]
    fn merge_configs_test() {
        let mut config = ConfigService {
            // service monitor
            enable_monitoring_service: None,
            api_tests: None,
            frontend_tests: None,
            ssl_tests: None,
            website_monitoring_interval: None,
            pause_reminder_interval: None,
            times_to_retry_after_error: None,
            // telegram_service
            enable_telegram_bot_commands: None,
            retrieve_commands_interval: None,
            telegram_bot_token: Some("asdfasdf.asdfasdf".to_string()),
            groups: None,
            // api
            api_host: None,
            api_port: None,
            api_token: None,
            enable_api: None,
        };

        config = ConfigService::merge_configs_with_defalt(config);

        // service monitor
        assert!(config.enable_monitoring_service.is_some());
        assert!(config.api_tests.is_some());
        assert!(config.frontend_tests.is_some());
        assert!(config.ssl_tests.is_some());
        assert!(config.website_monitoring_interval.is_some());
        assert!(config.pause_reminder_interval.is_some());
        assert!(config.times_to_retry_after_error.is_some());
        // telegram_service
        assert!(config.enable_telegram_bot_commands.is_some());
        assert!(config.retrieve_commands_interval.is_some());
        assert!(config.telegram_bot_token.is_some());
        assert!(config.groups.is_some());
        // api
        assert!(config.api_host.is_some());
        assert!(config.api_port.is_some());
        assert!(config.api_token.is_some());
        assert!(config.enable_api.is_some());
    }
}

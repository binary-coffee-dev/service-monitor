use serde::Deserialize;
use std::env::current_dir;
use std::{fs::File, io::BufReader};

use crate::monitor::services::website_vitality_service::{Get, RouteTest};

#[derive(Deserialize, Debug, Clone)]
pub struct ConfigService {
    // service monitor
    pub enable_monitoring_service: Option<bool>,

    // telegram_service
    pub enable_telegram_bot_commands: Option<bool>,
    pub telegram_api_url: Option<String>,
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
                println!("Configuration files doesn't exist.");
            }
        };

        configs
    }

    fn merge_configs_with_defalt(mut config: ConfigService) -> ConfigService {
        let default = ConfigService::default();
        // service monitor
        config.enable_monitoring_service = config
            .enable_monitoring_service
            .or_else(|| default.enable_monitoring_service);
        config.api_tests = config.api_tests.or_else(|| default.api_tests);
        config.frontend_tests = config.frontend_tests.or_else(|| default.frontend_tests);
        config.website_monitoring_interval = config
            .website_monitoring_interval
            .or_else(|| default.website_monitoring_interval);
        config.website_monitoring_interval = config
            .website_monitoring_interval
            .or_else(|| default.website_monitoring_interval);
        config.ssl_tests = config.ssl_tests.or_else(|| default.ssl_tests);
        config.pause_reminder_interval = config
            .pause_reminder_interval
            .or_else(|| default.pause_reminder_interval);
        config.times_to_retry_after_error = config
            .times_to_retry_after_error
            .or_else(|| default.times_to_retry_after_error);

        // telegram_service
        config.enable_telegram_bot_commands = config
            .enable_telegram_bot_commands
            .or_else(|| default.enable_telegram_bot_commands);
        config.retrieve_commands_interval = config
            .retrieve_commands_interval
            .or_else(|| default.retrieve_commands_interval);
        config.groups = config.groups.or_else(|| default.groups);

        // api
        config.api_host = config.api_host.or_else(|| default.api_host);
        config.api_port = config.api_port.or_else(|| default.api_port);
        config.api_token = config.api_token.or_else(|| default.api_token);
        config.enable_api = config.enable_api.or_else(|| default.enable_api);

        config
    }

    fn default() -> ConfigService {
        ConfigService {
            // service monitor
            enable_monitoring_service: Some(true),

            // telegram
            enable_telegram_bot_commands: Some(false),
            retrieve_commands_interval: Some(2),
            telegram_bot_token: None,
            groups: Some(Vec::new()),
            telegram_api_url: Some("https://api.telegram.org".to_string()),

            // api
            enable_api: Some(true),
            api_host: Some("0.0.0.0".to_string()),
            api_port: Some(5353),
            api_token: None,

            // general
            api_tests: Some(Vec::new()),
            frontend_tests: Some(Vec::new()),
            website_monitoring_interval: Some(20),
            ssl_tests: Some(Vec::new()),
            pause_reminder_interval: Some(86400),
            times_to_retry_after_error: Some(5),
        }
    }

    pub fn validate_configurations(&self) -> bool {
        self.telegram_bot_token.is_some()
            && (!self.enable_api.unwrap() || (self.enable_api.unwrap() && self.api_token.is_some()))
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
            telegram_api_url: Some("https://api.telegram.org".to_string()),

            // api
            enable_api: None,
            api_host: None,
            api_port: None,
            api_token: None,
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
        assert!(config.api_token.is_none());
        assert!(config.enable_api.is_some());
    }

    #[test]
    fn validate_configurations_test() {
        let mut config = ConfigService::default();

        // default config (telegram_bot_token is None)
        assert!(!config.validate_configurations());

        // if enabled_api is set api_token must be set
        config.telegram_bot_token = Some("asdfasdf.asdfasdf".to_string());
        config.enable_api = Some(true);
        assert!(!config.validate_configurations());

        //
    }
}

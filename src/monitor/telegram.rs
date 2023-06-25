use crate::config::Config;
use crate::monitor::telegram::models::{GetMyCommandsRes, GetUpdatesRes, SendMessageBody, Update};

use self::models::{BotCommand, SetMyCommandsBody};

pub mod models;

#[derive(Clone)]
pub struct Telegram {
    configs: Config,
    api_url: String,
}

impl Telegram {
    pub fn new(configs: Config) -> Telegram {
        let api_url = String::from(format!(
            "https://api.telegram.org/bot{}",
            configs.telegram_bot_token
        ));
        Telegram { configs, api_url }
    }

    pub async fn get_all_updates(&self) -> Vec<Update> {
        let mut updates_list = Vec::new();
        let mut offset = 0;
        let limit = 100;
        loop {
            let res = self.get_updates(limit, offset).await;
            if let Some(mut update_res) = res {
                if let Some(last) = update_res.result.last() {
                    offset = (last.update_id + 1) as usize;
                    updates_list.append(&mut update_res.result);
                    continue;
                }
            }
            break;
        }
        return updates_list;
    }

    async fn get_updates(&self, limit: usize, offset: usize) -> Option<GetUpdatesRes> {
        let route = String::from(format!(
            "{}/getUpdates?allowed_updates=[\"message\"]&limit={}&offset={}",
            self.api_url, limit, offset
        ));
        let res = Telegram::get_request(route).await;
        match res.status() {
            reqwest::StatusCode::OK => {
                let res_text = res.text().await.unwrap();
                let res_json: GetUpdatesRes = serde_json::from_str(&res_text)
                    .expect("Error deserializing configuration json file.");
                return Some(res_json);
            }
            _ => {}
        }
        return None;
    }

    pub async fn send_message(&self, text: String, groups: &Option<Vec<i64>>) {
        let groups_ids = if let Some(ids) = groups {
            ids.clone()
        } else {
            self.configs.groups.clone()
        };
        let route = String::from(format!("{}/sendMessage", self.api_url));
        println!("route: {}", route);
        for chat_id in groups_ids {
            let body_obj = SendMessageBody {
                chat_id,
                text: text.clone(),
            };
            let body = serde_json::to_string(&body_obj).expect("todo");
            println!("body: {}", body);
            let res =
                Telegram::post_request(route.clone(), body, String::from("application/json")).await;
            match res.status() {
                reqwest::StatusCode::OK => {}
                _ => {
                    println!(
                        "Error to send message to group: {}. res: {:?}",
                        chat_id, res
                    );
                }
            }
        }
    }

    pub fn services_sumary() {
        // todo: publish on telegram a subary of the services
    }

    pub async fn sync_commands(&self) {
        let configs = vec![
            BotCommand {
                command: "/check_all".to_string(),
                description: "Validate all.".to_string(),
            },
            BotCommand {
                command: "/check_api".to_string(),
                description: "Validate api.".to_string(),
            },
            BotCommand {
                command: "/check_frontend".to_string(),
                description: "Validate frontend.".to_string(),
            },
            BotCommand {
                command: "/check_certs".to_string(),
                description: "Validate certificates.".to_string(),
            },
            BotCommand {
                command: "/pause".to_string(),
                description: "Pause validations.".to_string(),
            },
            BotCommand {
                command: "/unpause".to_string(),
                description: "Unpause validations.".to_string(),
            },
        ];
        self.set_commands(configs).await;
    }

    pub async fn set_commands(&self, commands: Vec<BotCommand>) {
        let route = String::from(format!("{}/setMyCommands", self.api_url));
        let body_obj = SetMyCommandsBody { commands };
        let body = serde_json::to_string(&body_obj).expect("todo");
        let res = Telegram::post_request(route, body, String::from("application/json")).await;
        match res.status() {
            reqwest::StatusCode::OK => {}
            _ => {
                println!("Error setting up the list of commands.");
            }
        }
    }

    pub async fn get_commands(&self) -> Vec<BotCommand> {
        let route = String::from(format!("{}/getMyCommands", self.api_url));
        let bot_res = Telegram::get_request(route).await;
        match bot_res.status() {
            reqwest::StatusCode::OK => {
                let bot_commands_res = bot_res.text().await.unwrap();
                let bot_commands: GetMyCommandsRes =
                    serde_json::from_str(&bot_commands_res).expect("todo");
                return bot_commands.result;
            }
            _ => {}
        }
        return Vec::new();
    }

    pub async fn post_request(
        url: String,
        body: String,
        content_type: String,
    ) -> reqwest::Response {
        let client = reqwest::Client::new();

        return client
            .post(url.clone())
            .header("Content-Type", content_type)
            .body(body.clone())
            .send()
            .await
            .unwrap();
    }

    pub async fn get_request(url: String) -> reqwest::Response {
        let client = reqwest::Client::new();

        return client.get(url.to_owned()).send().await.unwrap();
    }
}

use crate::config::Config;
use crate::monitor::telegram::models::{GetMyCommandsRes, GetUpdatesRes, SendMessageBody, Update};

use self::models::{BotCommand, SetMyCommandsBody};

pub mod models;

#[derive(Clone)]
enum TelegramRequest {
    Post {
        url: String,
        body: String,
        content_type: String,
    },
    Get {
        url: String,
    },
}

#[derive(Clone)]
pub struct TelegramService {
    configs: Config,
    api_url: String,
    pending_messages: Vec<TelegramRequest>,
}

impl TelegramService {
    pub fn new(configs: Config) -> TelegramService {
        let api_url = String::from(format!(
            "https://api.telegram.org/bot{}",
            configs.telegram_bot_token.clone().unwrap()
        ));
        TelegramService {
            configs,
            api_url,
            pending_messages: Vec::new(),
        }
    }

    pub async fn get_all_updates(&mut self) -> Vec<Update> {
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

    async fn get_updates(&mut self, limit: usize, offset: usize) -> Option<GetUpdatesRes> {
        let route = String::from(format!(
            "{}/getUpdates?allowed_updates=[\"message\"]&limit={}&offset={}",
            self.api_url, limit, offset
        ));
        let res_value = self
            .retry_request(&TelegramRequest::Get { url: route })
            .await;
        if let Ok(res) = res_value {
            match res.status() {
                reqwest::StatusCode::OK => {
                    let res_text = res.text().await.unwrap();
                    let res_json: GetUpdatesRes = serde_json::from_str(&res_text)
                        .expect("Error deserializing configuration json file.");
                    return Some(res_json);
                }
                _ => {}
            }
        }
        return None;
    }

    pub async fn send_pendings_messages(&mut self) {
        let mut pendins: Vec<TelegramRequest> = Vec::new();
        self.pending_messages.push(TelegramRequest::Get {
            url: "some".to_string(),
        });
        while !self.pending_messages.is_empty() {
            pendins.push(self.pending_messages.remove(0));
        }
        for req in pendins.iter() {
            if let Ok(_) = self.retry_request(req).await {}
        }
    }

    pub async fn send_message(&mut self, text: String, groups: &Option<Vec<i64>>) {
        let groups_ids = if let Some(ids) = groups {
            ids.clone()
        } else {
            self.configs.groups.clone().unwrap()
        };
        let route = String::from(format!("{}/sendMessage", self.api_url));
        println!("route: {}", route);
        for chat_id in groups_ids {
            let body_obj = SendMessageBody {
                chat_id,
                text: text.clone(),
                parse_mode: "markdown".to_string(),
            };
            let body = serde_json::to_string(&body_obj).expect("todo");
            println!("body: {}", body);
            let res_value = self
                .retry_request(&TelegramRequest::Post {
                    url: route.clone(),
                    body,
                    content_type: String::from("application/json"),
                })
                .await;
            if let Ok(res) = res_value {
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
    }

    pub async fn sync_commands(&mut self) {
        let commands = vec![
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
        self.set_commands(commands).await;
    }

    pub async fn set_commands(&mut self, commands: Vec<BotCommand>) {
        let route = String::from(format!("{}/setMyCommands", self.api_url));
        let body_obj = SetMyCommandsBody { commands };
        let body = serde_json::to_string(&body_obj).expect("todo");
        let res_value = self
            .retry_request(&TelegramRequest::Post {
                url: route,
                body,
                content_type: String::from("application/json"),
            })
            .await;
        if let Ok(res) = res_value {
            match res.status() {
                reqwest::StatusCode::OK => {}
                _ => {
                    println!("Error setting up the list of commands.");
                }
            }
        }
    }

    pub async fn get_commands(&mut self) -> Vec<BotCommand> {
        let route = String::from(format!("{}/getMyCommands", self.api_url));
        let res = self
            .retry_request(&TelegramRequest::Get { url: route })
            .await;
        if let Ok(bot_res) = res {
            match bot_res.status() {
                reqwest::StatusCode::OK => {
                    let bot_commands_res = bot_res.text().await.unwrap();
                    let bot_commands: GetMyCommandsRes = serde_json::from_str(&bot_commands_res)
                        .expect("Error deserializing json response from string.");
                    return bot_commands.result;
                }
                _ => {}
            }
        }
        return Vec::new();
    }

    async fn retry_request(&mut self, req: &TelegramRequest) -> Result<reqwest::Response, String> {
        let mut times = 0;
        let times_to_retry_telegram = self.configs.times_to_retry.unwrap();
        loop {
            times += 1;
            match req {
                TelegramRequest::Post {
                    ref url,
                    ref body,
                    ref content_type,
                } => {
                    let res = self
                        .post_request(url.clone(), body.clone(), content_type.clone())
                        .await;
                    match res {
                        Ok(res_v) => {
                            return Ok(res_v);
                        }
                        Err(err) => {
                            if times >= times_to_retry_telegram {
                                self.pending_messages.push(TelegramRequest::Post {
                                    url: url.clone(),
                                    body: body.clone(),
                                    content_type: content_type.clone(),
                                });
                                return Err(format!(
                                    "Failing connecting to telegram api. {:?}",
                                    err
                                )
                                    .to_string());
                            }
                        }
                    }
                }
                TelegramRequest::Get { ref url } => {
                    let res = self.get_request(url.clone()).await;
                    match res {
                        Ok(res_v) => {
                            return Ok(res_v);
                        }
                        Err(err) => {
                            if times >= times_to_retry_telegram {
                                return Err(format!(
                                    "Failing connecting to telegram api. {:?}",
                                    err
                                )
                                    .to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    async fn post_request(
        &self,
        url: String,
        body: String,
        content_type: String,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let client = reqwest::Client::new();

        return client
            .post(url.clone())
            .header("Content-Type", content_type)
            .body(body.clone())
            .send()
            .await;
    }

    async fn get_request(&self, url: String) -> Result<reqwest::Response, reqwest::Error> {
        let client = reqwest::Client::new();
        return client.get(url.to_owned()).send().await;
    }
}

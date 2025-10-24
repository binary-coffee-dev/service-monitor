use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;
use tokio::time::sleep;

use crate::utils::ToMarkdown;
use crate::validator::Validator;

pub struct TelegramMonitor {
    pause_service: Arc<Mutex<bool>>,
    validator: Arc<Mutex<Validator>>,
}

impl TelegramMonitor {
    pub fn new(
        validator: Arc<Mutex<Validator>>,
        pause_service: Arc<Mutex<bool>>,
    ) -> TelegramMonitor {
        TelegramMonitor {
            pause_service,
            validator,
        }
    }

    pub async fn start_monitoring(&self) {
        TelegramMonitor::run_commands_sync(self).await;
        TelegramMonitor::run_telegram_monitor(self).await;
    }

    async fn run_commands_sync(&self) {
        self.validator.lock().await.sync_commands().await;
        let commands = self.validator.lock().await.get_commands().await;
        println!("commands: {:?}", commands);
    }

    async fn run_telegram_monitor(&self) {
        loop {
            self.validator.lock().await.send_pending_messages().await;
            let updates = self.validator.lock().await.get_all_updates().await;
            if !updates.is_empty() {
                println!("--------------------");
                println!("{:?}", updates);
            }
            for update in updates {
                if let Some(msg) = update.message {
                    if let Some(ent) = msg.entities {
                        let text = msg.text.unwrap();
                        let group_id = msg.chat.id;

                        for e in ent.iter() {
                            if e.type_value == String::from("bot_command") {
                                let offset_beg = e.offset as usize;
                                let offset_end = (e.offset + e.length) as usize;
                                let command_name = TelegramMonitor::extract_command(
                                    text.to_string()[offset_beg..offset_end].to_string(),
                                );

                                println!("command: {}", command_name);

                                match command_name.as_str() {
                                    "/check_all" => {
                                        self.validator
                                            .lock()
                                            .await
                                            .execute_check_api(group_id)
                                            .await;
                                        self.validator
                                            .lock()
                                            .await
                                            .execute_check_frontend(group_id)
                                            .await;
                                        self.validator
                                            .lock()
                                            .await
                                            .execute_check_certs(group_id)
                                            .await;
                                    }
                                    "/check_api" => {
                                        self.validator
                                            .lock()
                                            .await
                                            .execute_check_api(group_id)
                                            .await;
                                    }
                                    "/check_frontend" => {
                                        self.validator
                                            .lock()
                                            .await
                                            .execute_check_frontend(group_id)
                                            .await;
                                    }
                                    "/check_certs" => {
                                        self.validator
                                            .lock()
                                            .await
                                            .execute_check_certs(group_id)
                                            .await;
                                    }
                                    "/pause" => {
                                        let mut pause_v = self.pause_service.lock().await;
                                        *pause_v = true;
                                        self.validator.lock().await.send_telegram_message(
                                            "✅ Service is paused, if you want to reanudate it use the command /unpause.".to_string().parse_text_to_markdown(), &None
                                        ).await;
                                    }
                                    "/unpause" => {
                                        let mut pause_v = self.pause_service.lock().await;
                                        *pause_v = false;
                                        self.validator
                                            .lock()
                                            .await
                                            .send_telegram_message(
                                                "✅ Service is reanudated."
                                                    .to_string()
                                                    .parse_text_to_markdown(),
                                                &None,
                                            )
                                            .await;
                                    }
                                    _ => {
                                        println!("⚠️ Unknow command: {}", command_name);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            sleep(Duration::from_secs(2)).await;
        }
    }

    pub fn extract_command(command: String) -> String {
        if let Some(index) = command.find('@') {
            return command[0..index].to_string();
        }
        command
    }
}

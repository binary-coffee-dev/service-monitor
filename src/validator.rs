use std::sync::Arc;
use tokio::sync::Mutex;

use crate::monitor::telegram_service::models::BotCommand;
use crate::monitor::telegram_service::TelegramServiceTrait;
use crate::utils::ToMarkdown;
use crate::monitor::website_vitality_service::WebsiteVitalityService;

pub struct Validator {
    telegram: Arc<Mutex<dyn TelegramServiceTrait + Send>>,
    web: Arc<Mutex<WebsiteVitalityService>>,
}

impl Validator {
    pub fn new(
        telegram: Arc<Mutex<dyn TelegramServiceTrait + Send>>,
        web: Arc<Mutex<WebsiteVitalityService>>,
    ) -> Validator {
        Validator { telegram, web }
    }

    pub async fn execute_check_certs(&self, group_id: i64) {
        let errs = self.web.lock().await.certificates_vitality().await;
        self.handler_validation(
            errs,
            Some(
                "✅ Certificates are OK."
                    .to_string()
                    .parse_text_to_markdown(),
            ),
            Some(vec![group_id]),
        )
        .await;
    }

    pub async fn execute_check_frontend(&self, group_id: i64) {
        let errs = self.web.lock().await.frontend_vitality().await;
        self.handler_validation(
            errs,
            Some(
                "✅ Frontend is working fine."
                    .to_string()
                    .parse_text_to_markdown(),
            ),
            Some(vec![group_id]),
        )
        .await;
    }

    pub async fn execute_check_api(&self, group_id: i64) {
        let errs = self.web.lock().await.api_vitality().await;
        self.handler_validation(
            errs,
            Some(
                "✅ Api is working fine."
                    .to_string()
                    .parse_text_to_markdown(),
            ),
            Some(vec![group_id]),
        )
        .await;
    }

    pub async fn handler_validation(
        &self,
        errs: Vec<String>,
        success_msg: Option<String>,
        group_ids: Option<Vec<i64>>,
    ) {
        match success_msg {
            Some(msg) => {
                self.send_telegram_message(Validator::handler_errors(&errs, msg), &group_ids)
                    .await;
            }
            None => {
                if !errs.is_empty() {
                    self.send_telegram_message(
                        Validator::handler_errors(&errs, "".to_string()),
                        &group_ids,
                    )
                    .await;
                }
            }
        }
    }

    pub async fn send_telegram_message(&self, message: String, group_ids: &Option<Vec<i64>>) {
        self.telegram
            .lock()
            .await
            .send_message(message, group_ids)
            .await;
    }

    pub async fn send_pending_messages(&self) {
        self.telegram.lock().await.send_pendings_messages().await;
    }

    pub async fn get_commands(&self) -> Vec<BotCommand> {
        self.telegram.lock().await.get_commands().await
    }

    pub async fn sync_commands(&self) {
        self.telegram.lock().await.sync_commands().await;
    }

    pub async fn get_all_updates(&self) -> Vec<crate::monitor::telegram_service::models::Update> {
        self.telegram.lock().await.get_all_updates().await
    }

    pub async fn get_summary_website_errors(&self) -> Vec<String> {
        self.web.lock().await.summary().await
    }

    fn handler_errors(errs: &Vec<String>, default: String) -> String {
        if errs.len() > 0 {
            let mut report = "".to_string();
            for err in errs {
                report.push_str(&err);
                report.push_str("\n");
            }
            return report;
        }
        default
    }
}

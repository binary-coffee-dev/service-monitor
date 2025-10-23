use std::sync::Arc;

use tokio::sync::Mutex;

use crate::monitor::telegram::TelegramServiceTrait;
use crate::monitor::website::WebsiteService;
use crate::monitor::utils::ToMarkdown;

pub struct Validator {
    telegram: Arc<Mutex<dyn TelegramServiceTrait + Send>>,
    web: Arc<Mutex<WebsiteService>>,
}

impl Validator {
    pub fn new(telegram: Arc<Mutex<dyn TelegramServiceTrait + Send>>, web: Arc<Mutex<WebsiteService>>) -> Validator {
        Validator { telegram, web }
    }

    pub async fn execute_check_certs(&self, group_id: i64) {
        let errs = self.web.lock().await.certificates_vitaly().await;
        self.handler_validation(
            errs,
            Some("✅ Certificates are OK.".to_string().parse_text_to_markdown()),
            Some(vec![group_id]),
        ).await;
    }

    pub async fn execute_check_frontend(&self, group_id: i64) {
        let errs = self.web.lock().await.frontend_vitaly().await;
        self.handler_validation(
            errs,
            Some("✅ Frontend is working fine.".to_string().parse_text_to_markdown()),
            Some(vec![group_id]),
        ).await;
    }

    pub async fn execute_check_api(&self, group_id: i64) {
        let errs = self.web.lock().await.api_vitally().await;
        self.handler_validation(
            errs,
            Some("✅ Api is working fine.".to_string().parse_text_to_markdown()),
            Some(vec![group_id]),
        ).await;
    }

    pub async fn handler_validation(
        &self,
        errs: Vec<String>,
        success_msg: Option<String>,
        group_ids: Option<Vec<i64>>,
    ) {
        match success_msg {
            Some(msg) => {
                self.telegram.lock().await.send_message(Validator::handler_errors(&errs, msg), &group_ids)
                    .await;
            }
            None => {
                if !errs.is_empty() {
                    self.telegram.lock().await.send_message(Validator::handler_errors(&errs, "".to_string()), &group_ids)
                        .await;
                }
            }
        }
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

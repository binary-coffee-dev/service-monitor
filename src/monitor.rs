use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;
use tokio::time::sleep;

use crate::config::Config;
// use crate::monitor::api::start_server;
use crate::monitor::telegram::Telegram;
use crate::monitor::website::Website;

pub mod api;
pub mod telegram;
pub mod website;

pub struct Monitor {
    configs: Config,
}

impl Monitor {
    pub fn new(configs: Config) -> Monitor {
        Monitor { configs }
    }

    pub async fn start(&self) {
        let pause = Arc::new(Mutex::new(false));
        let rt = tokio::runtime::Runtime::new().unwrap();

        let config_ref_rm = self.configs.clone();
        let pause_ref_rm = pause.clone();
        let telegram_monitor_thread = rt.spawn(async move {
            let tel = Telegram::new(config_ref_rm.clone());
            let web = Website::new(config_ref_rm.clone());

            Monitor::run_commands_sync(&tel).await;
            Monitor::run_telegram_monitor(&tel, &web, pause_ref_rm).await;
        });

        let config_ref_wm = self.configs.clone();
        let pause_ref_wm = pause.clone();
        let website_monitor = rt.spawn(async move {
            let tel = Telegram::new(config_ref_wm.clone());
            let web = Website::new(config_ref_wm.clone());

            Monitor::run_website_monitor(
                &tel,
                &web,
                config_ref_wm.website_monitor_timeout.unwrap(),
                pause_ref_wm,
            )
            .await;
        });

        // start_server().unwrap();

        let _result = tokio::join!(telegram_monitor_thread, website_monitor);
    }

    async fn run_commands_sync(tel: &Telegram) {
        tel.sync_commands().await;
        let commands = tel.get_commands().await;
        println!("commands: {:?}", commands);
    }

    async fn run_telegram_monitor(tel: &Telegram, web: &Website, pause: Arc<Mutex<bool>>) {
        loop {
            let updates = tel.get_all_updates().await;
            if !updates.is_empty() {
                println!("--------------------");
                println!("{:?}", updates);
            }
            for update in updates {
                if let Some(msg) = update.message {
                    let text = msg.text.unwrap();
                    let group_id = msg.chat.id;
                    if let Some(ent) = msg.entities {
                        for e in ent.iter() {
                            if e.type_value == String::from("bot_command") {
                                let offset_beg = e.offset as usize;
                                let offset_end = (e.offset + e.length) as usize;
                                let command_name =
                                    text.to_string()[offset_beg..offset_end].to_string();
                                println!("command: {}", command_name);

                                // replace @monitor_bc_bot by a configuration attribute to make it
                                // modular
                                match command_name.as_str() {
                                    "/check_all" | "/check_all@monitor_bc_bot" => {
                                        Monitor::execute_check_api(tel, web, group_id).await;
                                        Monitor::execute_check_frontend(tel, web, group_id).await;
                                        Monitor::execute_check_certs(tel, web, group_id).await;
                                    }
                                    "/check_api" | "/check_api@monitor_bc_bot" => {
                                        Monitor::execute_check_api(tel, web, group_id).await;
                                    }
                                    "/check_frontend" | "/check_frontend@monitor_bc_bot" => {
                                        Monitor::execute_check_frontend(tel, web, group_id).await;
                                    }
                                    "/check_certs" | "/check_certs@monitor_bc_bot" => {
                                        Monitor::execute_check_certs(tel, web, group_id).await;
                                    }
                                    "/pause" | "/pause@monitor_bc_bot" => {
                                        let mut pause_v = pause.lock().await;
                                        *pause_v = true;
                                        tel.send_message("Service is paused, if you want to reanudate it use the command /unpause.".to_string(), &None).await;
                                    }
                                    "/unpause" | "/unpause@monitor_bc_bot" => {
                                        let mut pause_v = pause.lock().await;
                                        *pause_v = false;
                                        tel.send_message("Service is reanudated.".to_string(), &None).await;
                                    }
                                    _ => {
                                        println!("Unknow command: {}", command_name);
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

    async fn run_website_monitor(
        tel: &Telegram,
        web: &Website,
        timeout: u64,
        pause: Arc<Mutex<bool>>,
    ) {
        loop {
            let pause_v = pause.lock().await;
            if !*pause_v {
                let errors = web.sumary().await;

                if !errors.is_empty() {
                    for err in errors.iter() {
                        println!("Err: {}", err);
                    }

                    Monitor::handler_validation(errors, None, None, &tel).await;
                }
            }

            sleep(Duration::from_secs(timeout)).await;
        }
    }

    async fn execute_check_certs(tel: &Telegram, web: &Website, group_id: i64) {
        let errs = web.certificates_vitaly().await;
        Monitor::handler_validation(
            errs,
            Some("✅ Certificates are OK.".to_string()),
            Some(vec![group_id]),
            tel,
        )
        .await;
    }

    async fn execute_check_frontend(tel: &Telegram, web: &Website, group_id: i64) {
        let errs = web.frontend_vitaly().await;
        Monitor::handler_validation(
            errs,
            Some("✅ Frontend is working fine.".to_string()),
            Some(vec![group_id]),
            tel,
        )
        .await;
    }

    async fn execute_check_api(tel: &Telegram, web: &Website, group_id: i64) {
        let errs = web.api_vitaly().await;
        Monitor::handler_validation(
            errs,
            Some("✅ Api is working fine.".to_string()),
            Some(vec![group_id]),
            tel,
        )
        .await;
    }

    async fn handler_validation(
        errs: Vec<String>,
        success_msg: Option<String>,
        group_ids: Option<Vec<i64>>,
        tel: &Telegram,
    ) {
        match success_msg {
            Some(msg) => {
                tel.send_message(Monitor::handler_errors(&errs, msg), &group_ids)
                    .await;
            }
            None => {
                if !errs.is_empty() {
                    tel.send_message(Monitor::handler_errors(&errs, "".to_string()), &group_ids)
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
        return default;
    }
}

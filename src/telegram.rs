#[derive(Clone)]
pub struct Telegram {
    bot_token: String,
}

impl Telegram {
    pub fn new() -> Telegram {
        Telegram {
            bot_token: String::from(""),
        }
    }

    pub fn publish_message(&self) {
        // todo: send message to telegram
    }

    pub fn services_sumary() {
        // todo: publish on telegram a subary of the services
    }
}

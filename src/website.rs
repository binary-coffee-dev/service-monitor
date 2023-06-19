use crate::telegram::Telegram;

pub struct Website {
    api_url: String,
    routes_to_check: Vec<String>,
    frontend_url: String,

    telegram: Telegram,
}

impl Website {
    pub fn new(telegram: Telegram) -> Website {
        Website {
            api_url: String::from(""),
            routes_to_check: Vec::new(),
            frontend_url: String::from(""),
            telegram,
        }
    }

    pub fn sumary(&self) {
        self.api_vitaly();
        self.frontend_vitaly();
        self.certificates_vitaly();
    }

    pub fn api_vitaly(&self) {}

    pub fn frontend_vitaly(&self) {}

    pub fn certificates_vitaly(&self) {}
}

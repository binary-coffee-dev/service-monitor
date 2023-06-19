pub struct Website {
    api_url: String,
    routes_to_check: Vec<String>,
    frontend_url: String,
}

impl Website {
    pub fn new() {
        Website {
            api_url: String::from(""),
            routes_to_check: Vec::new(),
            frontend_url: String::from(""),
        }
    }

    pub fn api_vitaly(&self) {}

    pub fn frontend_vitaly(&self) {}

    pub fn certificates_vitaly(&self) {}
}

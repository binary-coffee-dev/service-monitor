use std::time::Duration;
use serde::{Deserialize, Serialize};

use checkssl::CheckSSL;
use reqwest::Client;

use crate::config::Config;
use crate::monitor::utils::ToMarkdown;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Post {
    pub url: String,
    pub body: String,
    pub content_type: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Get {
    pub url: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum RouteTest {
    POST(Post),
    GET(Get),
}

pub struct WebsiteService {
    configs: Config,
}

impl WebsiteService {
    pub fn new(configs: Config) -> WebsiteService {
        WebsiteService { configs }
    }

    pub async fn summary(&self) -> Vec<String> {
        let mut errors = Vec::new();

        errors.append(&mut self.api_vitally().await);
        errors.append(&mut self.frontend_vitaly().await);
        errors.append(&mut self.certificates_vitaly().await);

        errors
    }

    pub async fn api_vitally(&self) -> Vec<String> {
        let mut ret = Vec::new();
        let client = Client::new();
        if let Some(ref api_tests) = self.configs.api_tests {
            for test in api_tests.iter() {
                self.make_request(&test, &client, &mut ret).await;
            }
        }
        ret
    }

    pub async fn frontend_vitaly(&self) -> Vec<String> {
        let mut ret = Vec::new();
        let client = Client::new();
        if let Some(ref frontend_tests) = self.configs.frontend_tests {
            for test in frontend_tests.iter() {
                self.make_request(&test, &client, &mut ret).await;
            }
        }
        ret
    }

    pub async fn certificates_vitaly(&self) -> Vec<String> {
        let mut ret = Vec::new();
        if let Some(ref ssl_tests) = self.configs.ssl_tests {
            for get in ssl_tests.iter() {
                let Get { url } = get;

                match CheckSSL::from_domain(url.as_str()) {
                    Ok(_cert) => {
                        println!("Cert for url [{}] is ok.", url);
                    }
                    Err(_) => {
                        let msg = format!("❌ Error with cert, url: {}.", url).parse_text_to_markdown();
                        println!("{msg}");
                        ret.push(msg);
                    }
                };
            }
        }
        ret
    }

    async fn make_request(&self, test: &RouteTest, client: &Client, ret: &mut Vec<String>) {
        match test {
            RouteTest::POST(post) => {
                self.post_request(post, client, ret).await;
            }
            RouteTest::GET(get) => {
                self.get_request(get, client, ret).await;
            }
        }
    }

    async fn post_request(&self, post: &Post, client: &Client, ret: &mut Vec<String>) {
        let Post {
            url,
            body,
            content_type,
        } = post;
        let times_to_retry = self.configs.times_to_retry.unwrap();
        let mut times = 0;
        loop {
            times += 1;
            let res_value = client
                .post(url.clone())
                .header("Content-Type", content_type)
                .body(body.to_owned())
                .timeout(Duration::new(5, 0))
                .send()
                .await;

            match res_value {
                Ok(res) => match res.status() {
                    reqwest::StatusCode::OK => {
                        println!("Url POST [{}] is OK.", url);
                        break;
                    }
                    _ => {
                        ret.push(format!(
                            "❌ The url POST [{}] fails and return an status {}.",
                            url,
                            res.status()
                        ).parse_text_to_markdown());
                        break;
                    }
                },
                Err(err) => {
                    if times >= times_to_retry {
                        print!("Error: {:?}", err);
                        ret.push(format!("❌ The url POST [{}] fails.", url, ).parse_text_to_markdown());
                        break;
                    }
                }
            }
        }
    }

    async fn get_request(&self, get: &Get, client: &Client, ret: &mut Vec<String>) {
        let Get { url } = get;
        let times_to_retry = self.configs.times_to_retry.unwrap();
        let mut times = 0;
        loop {
            times += 1;
            let res_value = client.get(url.to_owned()).send().await;
            match res_value {
                Ok(res) => match res.status() {
                    reqwest::StatusCode::OK => {
                        println!("Url GET [{}] is OK.", url);
                        break;
                    }
                    _ => {
                        ret.push(format!(
                            "❌ The url GET [{}] fails and return an status {}.",
                            url,
                            res.status()
                        ).parse_text_to_markdown());
                        break;
                    }
                },
                Err(err) => {
                    if times >= times_to_retry {
                        print!("Error: {:?}", err);
                        ret.push(format!("❌ The url GET [{}] fails.", url).parse_text_to_markdown());
                        break;
                    }
                }
            }
        }
    }
}

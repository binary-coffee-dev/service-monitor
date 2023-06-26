use serde::{Deserialize, Serialize};

use checkssl::CheckSSL;
use reqwest::Client;

use crate::config::Config;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Post {
    url: String,
    body: String,
    content_type: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Get {
    url: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum RouteTest {
    POST(Post),
    GET(Get),
}

pub struct Website {
    configs: Config,
}

impl Website {
    pub fn new(configs: Config) -> Website {
        Website { configs }
    }

    pub async fn sumary(&self) -> Vec<String> {
        let mut errors = Vec::new();

        errors.append(&mut self.api_vitaly().await);
        errors.append(&mut self.frontend_vitaly().await);
        errors.append(&mut self.certificates_vitaly().await);

        return errors;
    }

    pub async fn api_vitaly(&self) -> Vec<String> {
        let mut ret = Vec::new();
        let client = Client::new();
        for test in self.configs.api_tests.iter() {
            self.make_request(&test, &client, &mut ret).await;
        }
        return ret;
    }

    pub async fn frontend_vitaly(&self) -> Vec<String> {
        let mut ret = Vec::new();
        let client = Client::new();
        for test in self.configs.frontend_tests.iter() {
            self.make_request(&test, &client, &mut ret).await;
        }
        return ret;
    }

    pub async fn certificates_vitaly(&self) -> Vec<String> {
        let mut ret = Vec::new();
        for get in self.configs.ssl_tests.iter() {
            let Get { url } = get;

            match CheckSSL::from_domain(url.as_str()) {
                Ok(_cert) => {
                    println!("Cert for url [{}] is ok.", url);
                }
                Err(_) => {
                    let msg = format!("❌ Error with cert, url: {}.", url).to_string();
                    println!("{msg}");
                    ret.push(msg);
                }
            };
        }
        return ret;
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
        let res = client
            .post(url.clone())
            .header("Content-Type", content_type)
            .body(body.to_owned())
            .send()
            .await
            .unwrap();

        match res.status() {
            reqwest::StatusCode::OK => {
                println!("Url POST [{}] is OK.", url);
            }
            _ => {
                ret.push(format!(
                    "❌ The url POST [{}] fails and return an status {}.",
                    url,
                    res.status()
                ));
            }
        }
    }

    async fn get_request(&self, get: &Get, client: &Client, ret: &mut Vec<String>) {
        let Get { url } = get;
        let res = client.get(url.to_owned()).send().await.unwrap();
        match res.status() {
            reqwest::StatusCode::OK => {
                println!("Url GET [{}] is OK.", url);
            }
            _ => {
                ret.push(format!(
                    "❌ The url GET [{}] fails and return an status {}.",
                    url,
                    res.status()
                ));
            }
        }
    }
}

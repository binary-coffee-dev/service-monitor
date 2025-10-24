use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Clone)]
pub struct GetUpdatesRes {
    pub result: Vec<Update>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Update {
    pub update_id: i64,
    pub message: Option<Message>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Message {
    pub chat: Chat,
    pub text: Option<String>,
    pub entities: Option<Vec<MessageEntity>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Chat {
    pub id: i64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct MessageEntity {
    #[serde(rename = "type")]
    pub type_value: String,
    pub offset: i64,
    pub length: i64,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SendMessageBody {
    pub chat_id: i64,
    pub text: String,
    pub parse_mode: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SetMyCommandsBody {
    pub commands: Vec<BotCommand>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GetMyCommandsRes {
    ok: bool,
    pub result: Vec<BotCommand>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BotCommand {
    pub command: String,
    pub description: String,
}

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Message {
    pub id: String,
    pub ticket_id: String,
    pub sender_id: String,
    pub sender_name: Option<String>,
    pub content: String,
    pub created_at: String,
}

#[derive(Deserialize)]
pub struct SendMessageRequest {
    pub content: String,
}

#[derive(Deserialize)]
pub struct ListMessagesQuery {
    pub before: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WsMessage {
    pub msg_type: String,
    pub message: Option<Message>,
    pub user_id: Option<String>,
    pub user_name: Option<String>,
}

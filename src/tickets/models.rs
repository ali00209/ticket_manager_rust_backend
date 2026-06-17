use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Ticket {
    pub id: String,
    pub subject: String,
    pub description: String,
    pub status: String,
    pub priority: String,
    pub customer_id: String,
    pub customer_name: Option<String>,
    pub department_id: String,
    pub department_name: Option<String>,
    pub assigned_agent_id: Option<String>,
    pub assigned_agent_name: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Deserialize)]
pub struct UpdateTicketRequest {
    pub status: Option<String>,
    pub priority: Option<String>,
    pub assigned_agent_id: Option<String>,
    pub department_id: Option<String>,
}

#[derive(Deserialize)]
pub struct ListTicketsQuery {
    pub status: Option<String>,
    pub priority: Option<String>,
    pub department_id: Option<String>,
    pub assigned_agent_id: Option<String>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Serialize)]
pub struct TicketList {
    pub tickets: Vec<Ticket>,
    pub total: i64,
    pub page: i64,
    pub limit: i64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Attachment {
    pub id: String,
    pub filename: String,
    pub original_name: String,
    pub mime_type: String,
    pub file_size: i64,
    pub uploaded_by: String,
    pub created_at: String,
}

#[derive(Serialize)]
pub struct TicketWithAttachments {
    #[serde(flatten)]
    pub ticket: Ticket,
    pub attachments: Vec<Attachment>,
}

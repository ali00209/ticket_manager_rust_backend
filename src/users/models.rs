use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: String,
    pub email: String,
    pub full_name: String,
    pub role: String,
    pub department_id: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub password: String,
    pub full_name: String,
    pub role: String,
    pub department_id: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateUserRequest {
    pub email: Option<String>,
    pub full_name: Option<String>,
    pub role: Option<String>,
    pub department_id: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Deserialize)]
pub struct ListUsersQuery {
    pub role: Option<String>,
    pub department_id: Option<String>,
    pub is_active: Option<bool>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Serialize)]
pub struct UserList {
    pub users: Vec<User>,
    pub total: i64,
    pub page: i64,
    pub limit: i64,
}

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// Unified application error type.
///
/// Each variant maps to an HTTP status code and a user-facing message.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Not found")]
    NotFound,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden")]
    Forbidden,

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Internal server error")]
    Internal(#[from] anyhow::Error),

    #[error("Database error")]
    Database(#[from] sqlx::Error),

    #[error("JWT error")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("Email error")]
    Email(String),

    #[error("File storage error: {0}")]
    Storage(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            AppError::Forbidden => (StatusCode::FORBIDDEN, self.to_string()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Validation(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg.clone()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
            AppError::Internal(e) => {
                tracing::error!("Internal error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
            AppError::Database(e) => {
                tracing::error!("Database error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
            AppError::Jwt(e) => {
                tracing::error!("JWT error: {:?}", e);
                (StatusCode::UNAUTHORIZED, "Invalid token".to_string())
            }
            AppError::Email(msg) => {
                tracing::error!("Email error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to send email".to_string(),
                )
            }
            AppError::Storage(msg) => {
                tracing::error!("Storage error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "File storage error".to_string(),
                )
            }
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}

/// Helper type for handler results.
pub type AppResult<T> = Result<T, AppError>;

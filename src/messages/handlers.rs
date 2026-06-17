use axum::{
    extract::{Path, Query, State},
    Json,
};

use crate::{auth::jwt::Claims, error::AppError, state::AppState};

use super::models::*;

pub async fn list_messages(
    State(state): State<AppState>,
    axum::Extension(_claims): axum::Extension<Claims>,
    Path(ticket_id): Path<String>,
    Query(query): Query<ListMessagesQuery>,
) -> Result<Json<Vec<Message>>, AppError> {
    let limit = query.limit.unwrap_or(50).min(100);

    let messages = if let Some(before_id) = &query.before {
        sqlx::query_as::<_, Message>(
            r#"
            SELECT
                CAST(m.id AS TEXT) as id,
                CAST(m.ticket_id AS TEXT) as ticket_id,
                CAST(m.sender_id AS TEXT) as sender_id,
                u.full_name as sender_name,
                m.content,
                CAST(m.created_at AS TEXT) as created_at
            FROM messages m
            JOIN users u ON m.sender_id = u.id
            JOIN messages bm ON bm.id = $2::uuid
            WHERE m.ticket_id = $1::uuid
              AND m.created_at < bm.created_at
            ORDER BY m.created_at DESC
            LIMIT $3
            "#,
        )
        .bind(&ticket_id)
        .bind(before_id)
        .bind(limit)
        .fetch_all(&state.pool)
        .await?
    } else {
        sqlx::query_as::<_, Message>(
            "
            SELECT
                CAST(m.id AS TEXT) as id,
                CAST(m.ticket_id AS TEXT) as ticket_id,
                CAST(m.sender_id AS TEXT) as sender_id,
                u.full_name as sender_name,
                m.content,
                CAST(m.created_at AS TEXT) as created_at
            FROM messages m
            JOIN users u ON m.sender_id = u.id
            WHERE m.ticket_id = $1::uuid
            ORDER BY m.created_at DESC
            LIMIT $2
            ",
        )
        .bind(&ticket_id)
        .bind(limit)
        .fetch_all(&state.pool)
        .await?
    };

    Ok(Json(messages))
}

pub async fn send_message(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(ticket_id): Path<String>,
    Json(payload): Json<SendMessageRequest>,
) -> Result<Json<Message>, AppError> {
    if payload.content.trim().is_empty() {
        return Err(AppError::Validation(
            "Message content cannot be empty".to_string(),
        ));
    }

    let message_id: String = sqlx::query_scalar(
        r#"
        INSERT INTO messages (ticket_id, sender_id, content)
        VALUES ($1::uuid, $2::uuid, $3)
        RETURNING CAST(id AS TEXT) id
        "#,
    )
    .bind(&ticket_id)
    .bind(&claims.sub)
    .bind(&payload.content)
    .fetch_one(&state.pool)
    .await?;

    let message = sqlx::query_as::<_, Message>(
        "
        SELECT
            CAST(m.id AS TEXT) as id,
            CAST(m.ticket_id AS TEXT) as ticket_id,
            CAST(m.sender_id AS TEXT) as sender_id,
            u.full_name as sender_name,
            m.content,
            CAST(m.created_at AS TEXT) as created_at
        FROM messages m
        JOIN users u ON m.sender_id = u.id
        WHERE m.id = $1::uuid
        ",
    )
    .bind(&message_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Json(message))
}

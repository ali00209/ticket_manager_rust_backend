use std::collections::HashMap;
use std::sync::Arc;

use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use tokio::sync::{Mutex, RwLock};

use crate::{auth::jwt::validate_token, state::AppState};

#[derive(Clone, Default)]
pub struct TicketConnections {
    connections: Arc<RwLock<HashMap<String, Vec<(String, tokio::sync::mpsc::UnboundedSender<String>)>>>>,
}

impl TicketConnections {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add(
        &self,
        ticket_id: &str,
        user_id: &str,
        sender: tokio::sync::mpsc::UnboundedSender<String>,
    ) {
        let mut conns = self.connections.write().await;
        conns
            .entry(ticket_id.to_string())
            .or_insert_with(Vec::new)
            .push((user_id.to_string(), sender));
    }

    pub async fn remove(&self, ticket_id: &str, user_id: &str) {
        let mut conns = self.connections.write().await;
        if let Some(connections) = conns.get_mut(ticket_id) {
            connections.retain(|(uid, _)| uid != user_id);
            if connections.is_empty() {
                conns.remove(ticket_id);
            }
        }
    }

    pub async fn broadcast(&self, ticket_id: &str, message: &str) {
        let conns = self.connections.read().await;
        if let Some(connections) = conns.get(ticket_id) {
            for (_, sender) in connections {
                let _ = sender.send(message.to_string());
            }
        }
    }
}

#[derive(Deserialize)]
struct WsIncoming {
    content: String,
}

#[derive(Serialize)]
struct WsOutgoing {
    msg_type: String,
    message_id: Option<String>,
    content: Option<String>,
    sender_id: Option<String>,
    sender_name: Option<String>,
    timestamp: Option<String>,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(ticket_id): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, ticket_id, state))
}

async fn handle_socket(socket: WebSocket, ticket_id: String, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    let token = match receiver.next().await {
        Some(Ok(Message::Text(text))) => text.to_string(),
        _ => {
            let _ = sender
                .send(Message::Text(
                    r#"{"msg_type":"error","message":"Authentication required"}"#.into(),
                ))
                .await;
            return;
        }
    };

    let claims = match validate_token(&token, &state.config.jwt_secret) {
        Ok(claims) if claims.token_type == crate::auth::jwt::TokenType::Access => claims,
        _ => {
            let _ = sender
                .send(Message::Text(
                    r#"{"msg_type":"error","message":"Invalid token"}"#.into(),
                ))
                .await;
            return;
        }
    };

    let user_id = claims.sub.clone();

    let welcome = WsOutgoing {
        msg_type: "connected".to_string(),
        message_id: None,
        content: None,
        sender_id: None,
        sender_name: None,
        timestamp: None,
    };
    let _ = sender
        .send(Message::Text(
            serde_json::to_string(&welcome).unwrap().into(),
        ))
        .await;

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();

    let user_name: String = sqlx::query_scalar::<_, String>(
        r#"SELECT full_name FROM users WHERE id = $1::uuid"#,
    )
    .bind(&user_id)
    .fetch_one(&state.pool)
    .await
    .unwrap_or_else(|_| "Unknown".to_string());

    state.ws_connections.add(&ticket_id, &user_id, tx).await;

    let join_msg = WsOutgoing {
        msg_type: "user_joined".to_string(),
        message_id: None,
        content: None,
        sender_id: Some(user_id.clone()),
        sender_name: Some(user_name.clone()),
        timestamp: None,
    };
    state
        .ws_connections
        .broadcast(&ticket_id, &serde_json::to_string(&join_msg).unwrap())
        .await;

    let sender = Arc::new(Mutex::new(sender));
    let sender_for_task = sender.clone();
    let rx_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let mut s = sender_for_task.lock().await;
            if s.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    while let Some(result) = receiver.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(_) => break,
        };

        match msg {
            Message::Text(text) => {
                let incoming: WsIncoming = match serde_json::from_str(&text) {
                    Ok(v) => v,
                    Err(_) => continue,
                };

                if incoming.content.trim().is_empty() {
                    continue;
                }

                let db_msg = sqlx::query(
                    r#"
                    INSERT INTO messages (ticket_id, sender_id, content)
                    VALUES ($1::uuid, $2::uuid, $3)
                    RETURNING CAST(id AS TEXT), CAST(created_at AS TEXT)
                    "#,
                )
                .bind(&ticket_id)
                .bind(&user_id)
                .bind(&incoming.content)
                .fetch_one(&state.pool)
                .await;

                if let Ok(db_msg) = db_msg {
                    if let (Ok(msg_id), Ok(created_at)) = (
                        db_msg.try_get::<String, _>("id"),
                        db_msg.try_get::<String, _>("created_at"),
                    ) {
                        let outgoing = WsOutgoing {
                            msg_type: "message".to_string(),
                            message_id: Some(msg_id),
                            content: Some(incoming.content),
                            sender_id: Some(user_id.clone()),
                            sender_name: Some(user_name.clone()),
                            timestamp: Some(created_at),
                        };
                        let json = serde_json::to_string(&outgoing).unwrap();
                        state.ws_connections.broadcast(&ticket_id, &json).await;
                    }
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    state.ws_connections.remove(&ticket_id, &user_id).await;

    let leave_msg = WsOutgoing {
        msg_type: "user_left".to_string(),
        message_id: None,
        content: None,
        sender_id: Some(user_id.clone()),
        sender_name: Some(user_name),
        timestamp: None,
    };
    state
        .ws_connections
        .broadcast(&ticket_id, &serde_json::to_string(&leave_msg).unwrap())
        .await;

    rx_task.abort();
}

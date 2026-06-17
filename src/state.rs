use sqlx::PgPool;

use crate::{config::Config, messages::websocket::TicketConnections};

/// Shared application state available to all handlers.
#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub pool: PgPool,
    pub ws_connections: TicketConnections,
}

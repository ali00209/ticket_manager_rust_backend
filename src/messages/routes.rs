use axum::routing::get;
use axum::Router;

use crate::state::AppState;

use super::handlers;

/// Build message routes.
///
/// - GET /    - List messages for a ticket (with pagination)
/// - POST /   - Send a message in a ticket's chat
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::list_messages).post(handlers::send_message))
}

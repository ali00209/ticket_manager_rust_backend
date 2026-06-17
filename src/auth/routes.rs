use axum::routing::post;
use axum::Router;

use crate::state::AppState;

use super::handlers;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/register", post(handlers::register))
        .route("/login", post(handlers::login))
        .route("/refresh", post(handlers::refresh))
}

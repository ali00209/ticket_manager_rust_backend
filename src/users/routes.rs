use axum::{routing::get, Router};

use crate::state::AppState;

use super::handlers;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::list_users).post(handlers::create_user))
        .route(
            "/:user_id",
            get(handlers::get_user)
                .patch(handlers::update_user)
                .delete(handlers::delete_user),
        )
}

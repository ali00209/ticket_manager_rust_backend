use axum::{routing::get, Router};

use crate::state::AppState;

use super::handlers;

/// Build department routes.
///
/// - GET /             - List all departments (any authenticated user)
/// - GET /:id          - Get department by ID (any authenticated user)
/// - POST /            - Create department (admin only)
/// - PATCH /:id        - Update department (admin only)
/// - GET /:id/agents   - List agents in department (any authenticated user)
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::list_departments).post(handlers::create_department))
        .route(
            "/:dept_id",
            get(handlers::get_department).patch(handlers::update_department),
        )
        .route("/:dept_id/agents", get(handlers::get_department_agents))
}

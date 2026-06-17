use axum::{routing::get, Router};

use crate::state::AppState;

use super::handlers;

/// Build ticket routes.
///
/// - GET /                  - List tickets (role-filtered)
/// - POST /                 - Create ticket (customer)
/// - GET /:id               - Get ticket by ID
/// - PATCH /:id             - Update ticket (agent/manager/admin)
/// - POST /:id/attachments  - Upload attachment to ticket
/// - GET /:id/attachments   - List attachments for ticket
pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/",
            get(handlers::list_tickets).post(handlers::create_ticket),
        )
        .route(
            "/:ticket_id",
            get(handlers::get_ticket).patch(handlers::update_ticket),
        )
        .route(
            "/:ticket_id/attachments",
            get(handlers::list_attachments).post(handlers::upload_attachment),
        )
}

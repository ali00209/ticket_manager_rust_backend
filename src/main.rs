use axum::{middleware, routing::get, Router};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use ticket_manager::{
    auth, config::Config, db, messages, notifications::jobs::EmailJobSender, state::AppState,
};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ticket_manager=debug,tower_http=debug".into()),
        )
        .init();

    let config = Config::from_env();
    let pool = db::create_pool(&config.database_url).await;

    // sqlx::migrate!()
    //     .run(&pool)
    //     .await
    //     .expect("Failed to run migrations");

    let ws_connections = messages::websocket::TicketConnections::new();
    let _email_sender = EmailJobSender::new(config.clone(), pool.clone());

    let state = AppState {
        config: config.clone(),
        pool: pool.clone(),
        ws_connections,
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let public_routes = Router::new()
        .route("/health", get(|| async { "OK" }))
        .nest("/auth", auth::routes::routes());

    let admin_routes = ticket_manager::users::routes::routes().layer(
        middleware::from_fn_with_state(state.clone(), auth::middleware::require_admin),
    );

    let protected_routes = Router::new()
        .nest("/users", admin_routes)
        .nest(
            "/departments",
            ticket_manager::departments::routes::routes(),
        )
        .nest("/tickets", ticket_manager::tickets::routes::routes())
        .nest("/tickets/:ticket_id/messages", messages::routes::routes())
        .route(
            "/ws/tickets/:ticket_id",
            get(messages::websocket::ws_handler),
        )
        .route("/admin/retry-jobs", get(retry_failed_jobs_handler))
        .route("/auth/me", get(auth::handlers::me))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::middleware::require_auth,
        ));

    let api_routes = public_routes.merge(protected_routes);

    let app = Router::new()
        .nest("/api", api_routes)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = config.server_addr();
    tracing::info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind");

    axum::serve(listener, app).await.expect("Server failed");
}

async fn retry_failed_jobs_handler(
    axum::Extension(claims): axum::Extension<ticket_manager::auth::jwt::Claims>,
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<axum::Json<serde_json::Value>, ticket_manager::error::AppError> {
    if claims.role != "admin" {
        return Err(ticket_manager::error::AppError::Forbidden);
    }

    let retried =
        ticket_manager::notifications::jobs::retry_failed_jobs(&state.config, &state.pool)
            .await
            .map_err(|e| ticket_manager::error::AppError::Internal(anyhow::anyhow!(e)))?;

    Ok(axum::Json(serde_json::json!({
        "message": format!("Retried {} failed jobs", retried),
        "retried": retried,
    })))
}

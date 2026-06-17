use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

/// Create a new PostgreSQL connection pool.
///
/// # Arguments
///
/// * `database_url` - PostgreSQL connection string
///
/// # Panics
///
/// Panics if the database connection cannot be established.
pub async fn create_pool(database_url: &str) -> PgPool {
    PgPoolOptions::new()
        .max_connections(10)
        .min_connections(1)
        .acquire_timeout(Duration::from_secs(30))
        .idle_timeout(Duration::from_secs(300))
        .connect(database_url)
        .await
        .expect("Failed to create database pool")
}

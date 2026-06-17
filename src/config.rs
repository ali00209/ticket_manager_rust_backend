use std::env;

/// Application configuration loaded from environment variables.
#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_access_expiry_hours: i64,
    pub jwt_refresh_expiry_days: i64,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_user: String,
    pub smtp_pass: String,
    pub smtp_from: String,
    pub upload_dir: String,
    pub server_host: String,
    pub server_port: u16,
}

impl Config {
    /// Load configuration from environment variables.
    ///
    /// # Panics
    ///
    /// Panics if required environment variables are not set.
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set"),
            jwt_secret: env::var("JWT_SECRET")
                .expect("JWT_SECRET must be set"),
            jwt_access_expiry_hours: env::var("JWT_ACCESS_EXPIRY_HOURS")
                .unwrap_or_else(|_| "1".to_string())
                .parse()
                .expect("JWT_ACCESS_EXPIRY_HOURS must be a number"),
            jwt_refresh_expiry_days: env::var("JWT_REFRESH_EXPIRY_DAYS")
                .unwrap_or_else(|_| "7".to_string())
                .parse()
                .expect("JWT_REFRESH_EXPIRY_DAYS must be a number"),
            smtp_host: env::var("SMTP_HOST")
                .unwrap_or_else(|_| "127.0.0.1".to_string()),
            smtp_port: env::var("SMTP_PORT")
                .unwrap_or_else(|_| "1025".to_string())
                .parse()
                .expect("SMTP_PORT must be a number"),
            smtp_user: env::var("SMTP_USER")
                .unwrap_or_default(),
            smtp_pass: env::var("SMTP_PASS")
                .unwrap_or_default(),
            smtp_from: env::var("SMTP_FROM")
                .unwrap_or_else(|_| "noreply@ticketmanager.dev".to_string()),
            upload_dir: env::var("UPLOAD_DIR")
                .unwrap_or_else(|_| "./uploads".to_string()),
            server_host: env::var("SERVER_HOST")
                .unwrap_or_else(|_| "0.0.0.0".to_string()),
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .expect("SERVER_PORT must be a number"),
        }
    }

    /// Returns the server address string (e.g., "0.0.0.0:8080").
    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.server_host, self.server_port)
    }
}

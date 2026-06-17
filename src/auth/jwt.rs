use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

/// JWT claims structure.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub role: String,
    pub exp: i64,
    pub token_type: TokenType,
}

/// Token type区分 access and refresh tokens.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum TokenType {
    Access,
    Refresh,
}

/// Generate an access token for a user.
///
/// # Arguments
///
/// * `user_id` - The user's UUID
/// * `email` - The user's email
/// * `role` - The user's role
/// * `secret` - JWT signing secret
/// * `expiry_hours` - Token expiry in hours
pub fn generate_access_token(
    user_id: &str,
    email: &str,
    role: &str,
    secret: &str,
    expiry_hours: i64,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let expires_at = now + Duration::hours(expiry_hours);

    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        role: role.to_string(),
        exp: expires_at.timestamp(),
        token_type: TokenType::Access,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

/// Generate a refresh token for a user.
///
/// # Arguments
///
/// * `user_id` - The user's UUID
/// * `email` - The user's email
/// * `role` - The user's role
/// * `secret` - JWT signing secret
/// * `expiry_days` - Token expiry in days
pub fn generate_refresh_token(
    user_id: &str,
    email: &str,
    role: &str,
    secret: &str,
    expiry_days: i64,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let expires_at = now + Duration::days(expiry_days);

    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        role: role.to_string(),
        exp: expires_at.timestamp(),
        token_type: TokenType::Refresh,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

/// Validate and decode a JWT token.
pub fn validate_token(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;
    Ok(token_data.claims)
}

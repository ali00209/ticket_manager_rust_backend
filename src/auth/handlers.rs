use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row};

use crate::{
    auth::jwt::{generate_access_token, generate_refresh_token, validate_token, TokenType},
    error::AppError,
    state::AppState,
};

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub full_name: String,
}

#[derive(Deserialize, Debug)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user: UserInfo,
}

#[derive(Serialize, FromRow, Debug)]
pub struct UserInfo {
    pub id: String,
    pub email: String,
    pub full_name: String,
    pub role: String,
}

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    if payload.email.is_empty() || payload.password.is_empty() || payload.full_name.is_empty() {
        return Err(AppError::Validation("All fields are required".to_string()));
    }

    if payload.password.len() < 8 {
        return Err(AppError::Validation(
            "Password must be at least 8 characters".to_string(),
        ));
    }

    let password_hash = bcrypt::hash(&payload.password, 12)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to hash password: {}", e)))?;

    let user = sqlx::query_as::<_, UserInfo>(
        "
        INSERT INTO users (email, password_hash, full_name, role)
        VALUES ($1, $2, $3, 'customer')
        RETURNING CAST(id AS TEXT) id, email, full_name, CAST(role AS TEXT) role
        ",
    )
    .bind(&payload.email)
    .bind(&password_hash)
    .bind(&payload.full_name)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(ref db_err) if db_err.constraint() == Some("users_email_key") => {
            AppError::Conflict("Email already registered".to_string())
        }
        _ => AppError::Database(e),
    })?;

    let access_token = generate_access_token(
        &user.id,
        &user.email,
        &user.role,
        &state.config.jwt_secret,
        state.config.jwt_access_expiry_hours,
    )?;

    let refresh_token = generate_refresh_token(
        &user.id,
        &user.email,
        &user.role,
        &state.config.jwt_secret,
        state.config.jwt_refresh_expiry_days,
    )?;

    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        user,
    }))
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let row = sqlx::query(
        r#"
        SELECT
            CAST(id AS TEXT) as id,
            email,
            full_name,
            CAST(role AS TEXT) as role,
            password_hash
        FROM users
        WHERE email = $1 AND is_active = true
        "#,
    )
    .bind(&payload.email)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::Unauthorized)?;

    let user = UserInfo {
        id: row.get("id"),
        email: row.get("email"),
        full_name: row.get("full_name"),
        role: row.get("role"),
    };
    let password_hash: String = row.get("password_hash");

    let is_valid = bcrypt::verify(&payload.password, &password_hash)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to verify password: {}", e)))?;

    if !is_valid {
        return Err(AppError::Unauthorized);
    }

    let access_token = generate_access_token(
        &user.id,
        &user.email,
        &user.role,
        &state.config.jwt_secret,
        state.config.jwt_access_expiry_hours,
    )?;

    let refresh_token = generate_refresh_token(
        &user.id,
        &user.email,
        &user.role,
        &state.config.jwt_secret,
        state.config.jwt_refresh_expiry_days,
    )?;

    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        user,
    }))
}

pub async fn refresh(
    State(state): State<AppState>,
    Json(payload): Json<RefreshRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let claims = validate_token(&payload.refresh_token, &state.config.jwt_secret)?;

    if claims.token_type != TokenType::Refresh {
        return Err(AppError::Unauthorized);
    }

    let user = sqlx::query_as::<_, UserInfo>(
        r#"
        SELECT CAST(id AS text) as id, email, full_name, CAST(role AS text) as role
        FROM users
        WHERE id = $1::uuid AND is_active = true
        "#,
    )
    .bind(&claims.sub)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::Unauthorized)?;

    let access_token = generate_access_token(
        &user.id,
        &user.email,
        &user.role,
        &state.config.jwt_secret,
        state.config.jwt_access_expiry_hours,
    )?;

    let refresh_token = generate_refresh_token(
        &user.id,
        &user.email,
        &user.role,
        &state.config.jwt_secret,
        state.config.jwt_refresh_expiry_days,
    )?;

    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        user,
    }))
}

pub async fn me(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<crate::auth::jwt::Claims>,
) -> Result<Json<UserInfo>, AppError> {
    let user = sqlx::query_as::<_, UserInfo>(
        r#"
        SELECT CAST(id AS text) as id, email, full_name, CAST(role AS text) as role
        FROM users
        WHERE id = $1::uuid AND is_active = true
        "#,
    )
    .bind(&claims.sub)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Json(user))
}

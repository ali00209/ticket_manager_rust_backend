use axum::{
    extract::{Request, State},
    http::HeaderMap,
    middleware::Next,
    response::Response,
};

use crate::{
    auth::jwt::validate_token,
    error::AppError,
    state::AppState,
};

/// Extract and validate JWT from the Authorization header.
///
/// Sets the user's claims in request extensions for downstream handlers.
pub async fn require_auth(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = extract_token_from_headers(request.headers())?;
    let claims = validate_token(&token, &state.config.jwt_secret)?;

    if claims.token_type != crate::auth::jwt::TokenType::Access {
        return Err(AppError::Unauthorized);
    }

    request.extensions_mut().insert(claims);
    Ok(next.run(request).await)
}

/// Middleware that requires the user to have admin role.
pub async fn require_admin(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = extract_token_from_headers(request.headers())?;
    let claims = validate_token(&token, &state.config.jwt_secret)?;

    if claims.token_type != crate::auth::jwt::TokenType::Access {
        return Err(AppError::Unauthorized);
    }

    if claims.role != "admin" {
        return Err(AppError::Forbidden);
    }

    request.extensions_mut().insert(claims);
    Ok(next.run(request).await)
}

/// Middleware that requires the user to have admin or manager role.
pub async fn require_admin_or_manager(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = extract_token_from_headers(request.headers())?;
    let claims = validate_token(&token, &state.config.jwt_secret)?;

    if claims.token_type != crate::auth::jwt::TokenType::Access {
        return Err(AppError::Unauthorized);
    }

    if claims.role != "admin" && claims.role != "manager" {
        return Err(AppError::Forbidden);
    }

    request.extensions_mut().insert(claims);
    Ok(next.run(request).await)
}

/// Extract Bearer token from Authorization header.
fn extract_token_from_headers(headers: &HeaderMap) -> Result<String, AppError> {
    let auth_header = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::Unauthorized)?;

    auth_header
        .strip_prefix("Bearer ")
        .map(|s| s.to_string())
        .ok_or(AppError::Unauthorized)
}

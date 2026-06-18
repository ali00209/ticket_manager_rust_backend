use axum::{
    extract::{Path, Query, State},
    Json,
};

use crate::{auth::jwt::Claims, error::AppError, state::AppState};

use super::models::*;

pub async fn list_users(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Query(query): Query<ListUsersQuery>,
) -> Result<Json<UserList>, AppError> {
    if claims.role != "superadmin" {
        return Err(AppError::Forbidden);
    }

    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(20).min(100);
    let offset = (page - 1) * limit;

    let has_role = query.role.is_some();
    let has_dept = query.department_id.is_some();
    let has_active = query.is_active.is_some();

    let mut bind_idx = 1;
    let role_clause = if has_role {
        let idx = bind_idx;
        bind_idx += 1;
        format!("AND CAST(role AS TEXT) = ${}", idx)
    } else {
        String::new()
    };
    let dept_clause = if has_dept {
        let idx = bind_idx;
        bind_idx += 1;
        format!("AND department_id = ${}::uuid", idx)
    } else {
        String::new()
    };
    let active_clause = if has_active {
        let idx = bind_idx;
        bind_idx += 1;
        format!("AND is_active = ${}", idx)
    } else {
        String::new()
    };

    let sql = format!(
        r#"
        SELECT
            CAST(id AS TEXT) as id,
            email,
            full_name,
            CAST(role AS TEXT) as role,
            CAST(department_id AS TEXT) as department_id,
            is_active,
            CAST(created_at AS TEXT) as created_at,
            CAST(updated_at AS TEXT) as updated_at
        FROM users
        WHERE 1=1 {} {} {}
        ORDER BY created_at DESC
        LIMIT ${} OFFSET ${}
        "#,
        role_clause,
        dept_clause,
        active_clause,
        bind_idx,
        bind_idx + 1,
    );

    let mut q = sqlx::query_as::<_, User>(&sql);
    if has_role {
        q = q.bind(&query.role);
    }
    if has_dept {
        q = q.bind(&query.department_id);
    }
    if has_active {
        q = q.bind(query.is_active);
    }
    let users = q.bind(limit).bind(offset).fetch_all(&state.pool).await?;

    let mut count_sql = "SELECT COUNT(*)::bigint FROM users WHERE 1=1".to_string();
    let mut count_bind_idx = 1;
    if has_role {
        count_sql.push_str(&format!(" AND CAST(role AS TEXT) = ${}", count_bind_idx));
        count_bind_idx += 1;
    }
    if has_dept {
        count_sql.push_str(&format!(" AND department_id = ${}::uuid", count_bind_idx));
        count_bind_idx += 1;
    }
    if has_active {
        count_sql.push_str(&format!(" AND is_active = ${}", count_bind_idx));
    }

    let mut cq = sqlx::query_scalar::<_, i64>(&count_sql);
    if has_role {
        cq = cq.bind(&query.role);
    }
    if has_dept {
        cq = cq.bind(&query.department_id);
    }
    if has_active {
        cq = cq.bind(query.is_active);
    }
    let total = cq.fetch_one(&state.pool).await?;

    Ok(Json(UserList {
        users,
        total,
        page,
        limit,
    }))
}

pub async fn get_user(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(user_id): Path<String>,
) -> Result<Json<User>, AppError> {
    if claims.role != "admin" {
        return Err(AppError::Forbidden);
    }

    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT
            CAST(id AS TEXT) as id,
            email,
            full_name,
            CAST(role AS TEXT) as role,
            CAST(department_id AS TEXT) as department_id,
            is_active,
            CAST(created_at AS TEXT) as created_at,
            CAST(updated_at AS TEXT) as updated_at
        FROM users
        WHERE id = $1::uuid
        "#,
    )
    .bind(&user_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Json(user))
}

pub async fn create_user(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<User>, AppError> {
    if claims.role != "superadmin" {
        return Err(AppError::Forbidden);
    }

    if payload.email.is_empty() || payload.password.is_empty() || payload.full_name.is_empty() {
        return Err(AppError::Validation("All fields are required".to_string()));
    }

    if payload.password.len() < 8 {
        return Err(AppError::Validation(
            "Password must be at least 8 characters".to_string(),
        ));
    }

    let valid_roles = ["customer", "agent", "deptadmin"];
    if !valid_roles.contains(&payload.role.as_str()) {
        return Err(AppError::Validation(format!(
            "Invalid role. Must be one of: {}",
            valid_roles.join(", ")
        )));
    }

    let password_hash = bcrypt::hash(&payload.password, 12)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to hash password: {}", e)))?;

    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (email, password_hash, full_name, role, department_id)
        VALUES ($1, $2, $3, $4::user_role, $5::uuid)
        RETURNING
            CAST(id AS TEXT) as id,
            email,
            full_name,
            CAST(role AS TEXT) as role,
            CAST(department_id AS TEXT) as department_id,
            is_active,
            CAST(created_at AS TEXT) as created_at,
            CAST(updated_at AS TEXT) as updated_at
        "#,
    )
    .bind(&payload.email)
    .bind(&password_hash)
    .bind(&payload.full_name)
    .bind(&payload.role)
    .bind(&payload.department_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(ref db_err) if db_err.constraint() == Some("users_email_key") => {
            AppError::Conflict("Email already exists".to_string())
        }
        _ => AppError::Database(e),
    })?;

    Ok(Json(user))
}

pub async fn update_user(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(user_id): Path<String>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<Json<User>, AppError> {
    if claims.role != "admin" {
        return Err(AppError::Forbidden);
    }

    let result = sqlx::query(
        r#"
        UPDATE users
        SET
            email = COALESCE($2, email),
            full_name = COALESCE($3, full_name),
            role = COALESCE($4::user_role, role),
            department_id = COALESCE($5::uuid, department_id),
            is_active = COALESCE($6, is_active),
            updated_at = NOW()
        WHERE id = $1::uuid
        "#,
    )
    .bind(&user_id)
    .bind(&payload.email)
    .bind(&payload.full_name)
    .bind(&payload.role)
    .bind(&payload.department_id)
    .bind(payload.is_active)
    .execute(&state.pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT
          CAST(id AS TEXT) as id,
            email,
            full_name,
            CAST(role AS TEXT) as role,
            CAST(department_id AS TEXT) as department_id,
            is_active,
            CAST(created_at AS TEXT) as created_at,
            CAST(updated_at AS TEXT) as updated_at
        FROM users
        WHERE id = $1::uuid
        "#,
    )
    .bind(&user_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(ref db_err) if db_err.constraint() == Some("users_email_key") => {
            AppError::Conflict("Email already exists".to_string())
        }
        _ => AppError::Database(e),
    })?;

    Ok(Json(user))
}

pub async fn delete_user(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(user_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    if claims.role != "admin" {
        return Err(AppError::Forbidden);
    }

    let result = sqlx::query(
        r#"
        UPDATE users SET is_active = false, updated_at = NOW()
        WHERE id = $1::uuid AND is_active = true
        "#,
    )
    .bind(&user_id)
    .execute(&state.pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(Json(serde_json::json!({
        "message": "User deactivated successfully"
    })))
}

use axum::{
    extract::{Path, State},
    Json,
};

use crate::{auth::jwt::Claims, error::AppError, state::AppState};

use super::models::*;

pub async fn list_departments(
    State(state): State<AppState>,
) -> Result<Json<Vec<Department>>, AppError> {
    let departments = sqlx::query_as::<_, Department>(
        r#"
        SELECT
            CAST(id AS text) as id,
            name,
            description,
            is_active,
            CAST(created_at AS text) as created_at,
            CAST(updated_at AS text) as updated_at
        FROM departments
        ORDER BY name
        "#,
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(departments))
}

pub async fn get_department(
    State(state): State<AppState>,
    Path(dept_id): Path<String>,
) -> Result<Json<Department>, AppError> {
    let department = sqlx::query_as::<_, Department>(
        r#"
        SELECT
            CAST(id AS text) as id,
            name,
            description,
            is_active,
            CAST(created_at AS text) as created_at,
            CAST(updated_at AS text) as updated_at
        FROM departments
        WHERE id = $1::uuid
        "#,
    )
    .bind(&dept_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Json(department))
}

pub async fn create_department(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Json(payload): Json<CreateDepartmentRequest>,
) -> Result<Json<Department>, AppError> {
    if claims.role != "admin" {
        return Err(AppError::Forbidden);
    }

    if payload.name.is_empty() {
        return Err(AppError::Validation("Name is required".to_string()));
    }

    let department = sqlx::query_as::<_, Department>(
        "
        INSERT INTO departments (name, description)
        VALUES ($1, $2)
        RETURNING
            CAST(id AS text) as id,
            name,
            description,
            is_active,
            CAST(created_at AS text) as created_at,
            CAST(updated_at AS text) as updated_at
        ",
    )
    .bind(&payload.name)
    .bind(&payload.description)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(ref db_err)
            if db_err.constraint() == Some("departments_name_key") =>
        {
            AppError::Conflict("Department name already exists".to_string())
        }
        _ => AppError::Database(e),
    })?;

    Ok(Json(department))
}

pub async fn update_department(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(dept_id): Path<String>,
    Json(payload): Json<UpdateDepartmentRequest>,
) -> Result<Json<Department>, AppError> {
    if claims.role != "admin" {
        return Err(AppError::Forbidden);
    }

    let result = sqlx::query(
        r#"
        UPDATE departments
        SET
            name = COALESCE($2, name),
            description = COALESCE($3, description),
            is_active = COALESCE($4, is_active),
            updated_at = NOW()
        WHERE id = $1::uuid
        "#,
    )
    .bind(&dept_id)
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(payload.is_active)
    .execute(&state.pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    let department = sqlx::query_as::<_, Department>(
        r#"
        SELECT
            CAST(id AS TEXT) as id,
            name,
            description,
            is_active,
            CAST(created_at AS TEXT) as created_at,
            CAST(updated_at AS TEXT) as updated_at
        FROM departments
        WHERE id = $1::uuid
        "#,
    )
    .bind(&dept_id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(department))
}

pub async fn get_department_agents(
    State(state): State<AppState>,
    Path(dept_id): Path<String>,
) -> Result<Json<Vec<DepartmentAgent>>, AppError> {
    let agents = sqlx::query_as::<_, DepartmentAgent>(
        r#"
        SELECT
            CAST(id AS TEXT) as id,
            email,
            full_name,
            is_active
        FROM users
        WHERE department_id = $1::uuid
          AND role IN ('agent', 'manager')
        ORDER BY full_name
        "#,
    )
    .bind(&dept_id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(agents))
}

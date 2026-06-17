use axum::{
    extract::{Multipart, Path, Query, State},
    Json,
};

use crate::{auth::jwt::Claims, error::AppError, state::AppState};

use super::models::*;

const TICKET_SELECT: &str = r#"
    SELECT
        CAST(t.id AS TEXT) as id,
        t.subject,
        t.description,
        CAST(t.status AS TEXT) as status,
        CAST(t.priority AS TEXT) as priority,
        CAST(t.customer_id AS TEXT) as customer_id,
        cu.full_name as customer_name,
        CAST(t.department_id AS TEXT) as department_id,
        d.name as department_name,
        CAST(t.assigned_agent_id AS TEXT) as assigned_agent_id,
        ag.full_name as assigned_agent_name,
        CAST(t.created_at AS TEXT) as created_at,
        CAST(t.updated_at AS TEXT) as updated_at
    FROM tickets t
    JOIN users cu ON t.customer_id = cu.id
    JOIN departments d ON t.department_id = d.id
    LEFT JOIN users ag ON t.assigned_agent_id = ag.id
"#;

async fn fetch_ticket(pool: &sqlx::PgPool, ticket_id: &str) -> Result<Ticket, AppError> {
    let query = format!("{} WHERE t.id = $1::uuid", TICKET_SELECT);
    sqlx::query_as::<_, Ticket>(&query)
        .bind(ticket_id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)
}

pub async fn list_tickets(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Query(query): Query<ListTicketsQuery>,
) -> Result<Json<TicketList>, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(20).min(100);
    let offset = (page - 1) * limit;

    let (tickets, total) = match claims.role.as_str() {
        "customer" => {
            let has_status = query.status.is_some();
            let has_priority = query.priority.is_some();
            let sql = format!(
                "{} WHERE t.customer_id = $1::uuid
                  {} {}
                ORDER BY t.created_at DESC
                LIMIT ${} OFFSET ${}",
                TICKET_SELECT,
                if has_status {
                    "AND CAST(t.status AS TEXT) = $2"
                } else {
                    ""
                },
                if has_priority {
                    format!(
                        "AND CAST(t.priority AS TEXT) = ${}",
                        if has_status { 3 } else { 2 }
                    )
                } else {
                    String::new()
                },
                if has_status && has_priority {
                    4
                } else if has_status || has_priority {
                    3
                } else {
                    2
                },
                if has_status && has_priority {
                    5
                } else if has_status || has_priority {
                    4
                } else {
                    3
                },
            );
            let mut q = sqlx::query_as::<_, Ticket>(&sql).bind(&claims.sub);
            if has_status {
                q = q.bind(&query.status);
            }
            if has_priority {
                q = q.bind(&query.priority);
            }
            let tickets = q.bind(limit).bind(offset).fetch_all(&state.pool).await?;

            let total: i64 = sqlx::query_scalar(
                "SELECT COUNT(*)::bigint FROM tickets WHERE customer_id = $1::uuid",
            )
            .bind(&claims.sub)
            .fetch_one(&state.pool)
            .await?;
            (tickets, total)
        }
        "agent" => {
            let has_status = query.status.is_some();
            let has_priority = query.priority.is_some();
            let sql = format!(
                "{} JOIN users u ON u.id = $1::uuid
                WHERE t.department_id = u.department_id
                  {} {}
                ORDER BY t.created_at DESC
                LIMIT ${} OFFSET ${}",
                TICKET_SELECT,
                if has_status {
                    "AND CAST(t.status AS TEXT) = $2"
                } else {
                    ""
                },
                if has_priority {
                    format!(
                        "AND CAST(t.priority AS TEXT) = ${}",
                        if has_status { 3 } else { 2 }
                    )
                } else {
                    String::new()
                },
                if has_status && has_priority {
                    4
                } else if has_status || has_priority {
                    3
                } else {
                    2
                },
                if has_status && has_priority {
                    5
                } else if has_status || has_priority {
                    4
                } else {
                    3
                },
            );
            let mut q = sqlx::query_as::<_, Ticket>(&sql).bind(&claims.sub);
            if has_status {
                q = q.bind(&query.status);
            }
            if has_priority {
                q = q.bind(&query.priority);
            }
            let tickets = q.bind(limit).bind(offset).fetch_all(&state.pool).await?;

            let total: i64 = sqlx::query_scalar(
                r#"
                SELECT COUNT(*)::bigint FROM tickets t
                JOIN users u ON u.id = $1::uuid
                WHERE t.department_id = u.department_id
                "#,
            )
            .bind(&claims.sub)
            .fetch_one(&state.pool)
            .await?;
            (tickets, total)
        }
        _ => {
            let has_dept = query.department_id.is_some();
            let has_status = query.status.is_some();
            let has_priority = query.priority.is_some();

            let mut bind_idx = 1;
            let dept_clause = if has_dept {
                let idx = bind_idx;
                bind_idx += 1;
                format!("AND t.department_id = ${}::uuid", idx)
            } else {
                String::new()
            };
            let status_clause = if has_status {
                let idx = bind_idx;
                bind_idx += 1;
                format!("AND CAST(t.status AS TEXT) = ${}", idx)
            } else {
                String::new()
            };
            let priority_clause = if has_priority {
                let idx = bind_idx;
                bind_idx += 1;
                format!("AND CAST(t.priority AS TEXT) = ${}", idx)
            } else {
                String::new()
            };

            let sql = format!(
                "{} WHERE 1=1 {} {} {}
                ORDER BY t.created_at DESC
                LIMIT ${} OFFSET ${}",
                TICKET_SELECT,
                dept_clause,
                status_clause,
                priority_clause,
                bind_idx,
                bind_idx + 1,
            );
            let mut q = sqlx::query_as::<_, Ticket>(&sql);
            if has_dept {
                q = q.bind(&query.department_id);
            }
            if has_status {
                q = q.bind(&query.status);
            }
            if has_priority {
                q = q.bind(&query.priority);
            }
            let tickets = q.bind(limit).bind(offset).fetch_all(&state.pool).await?;

            let mut count_sql = "SELECT COUNT(*)::bigint FROM tickets WHERE 1=1".to_string();
            let mut count_bind_idx = 1;
            if has_dept {
                count_sql.push_str(&format!(" AND department_id = ${}::uuid", count_bind_idx));
                count_bind_idx += 1;
            }
            if has_status {
                count_sql.push_str(&format!(" AND CAST(status AS TEXT) = ${}", count_bind_idx));
                count_bind_idx += 1;
            }
            if has_priority {
                count_sql.push_str(&format!(
                    " AND CAST(priority AS TEXT) = ${}",
                    count_bind_idx
                ));
            }

            let mut cq = sqlx::query_scalar::<_, i64>(&count_sql);
            if has_dept {
                cq = cq.bind(&query.department_id);
            }
            if has_status {
                cq = cq.bind(&query.status);
            }
            if has_priority {
                cq = cq.bind(&query.priority);
            }
            let total = cq.fetch_one(&state.pool).await?;
            (tickets, total)
        }
    };

    Ok(Json(TicketList {
        tickets,
        total,
        page,
        limit,
    }))
}

pub async fn get_ticket(
    State(state): State<AppState>,
    axum::Extension(_claims): axum::Extension<Claims>,
    Path(ticket_id): Path<String>,
) -> Result<Json<Ticket>, AppError> {
    let ticket = fetch_ticket(&state.pool, &ticket_id).await?;
    Ok(Json(ticket))
}

pub async fn create_ticket(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    mut multipart: Multipart,
) -> Result<Json<TicketWithAttachments>, AppError> {
    let mut subject: Option<String> = None;
    let mut description: Option<String> = None;
    let mut department_id: Option<String> = None;
    let mut priority: Option<String> = None;
    let mut file_data: Option<Vec<u8>> = None;
    let mut filename: Option<String> = None;
    let mut mime_type: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?
    {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "subject" => {
                let text = field
                    .text()
                    .await
                    .map_err(|e| AppError::BadRequest(e.to_string()))?;
                subject = Some(text);
            }
            "description" => {
                let text = field
                    .text()
                    .await
                    .map_err(|e| AppError::BadRequest(e.to_string()))?;
                description = Some(text);
            }
            "department_id" => {
                let text = field
                    .text()
                    .await
                    .map_err(|e| AppError::BadRequest(e.to_string()))?;
                department_id = Some(text);
            }
            "priority" => {
                let text = field
                    .text()
                    .await
                    .map_err(|e| AppError::BadRequest(e.to_string()))?;
                priority = Some(text);
            }
            "file" => {
                filename = Some(field.file_name().unwrap_or("unknown").to_string());
                mime_type = Some(
                    field
                        .content_type()
                        .unwrap_or("application/octet-stream")
                        .to_string(),
                );
                file_data = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|e| AppError::BadRequest(e.to_string()))?
                        .to_vec(),
                );
            }
            _ => {}
        }
    }

    let subject = subject.ok_or(AppError::Validation("Subject is required".to_string()))?;
    let description =
        description.ok_or(AppError::Validation("Description is required".to_string()))?;
    let department_id = department_id.ok_or(AppError::Validation(
        "Department ID is required".to_string(),
    ))?;

    if subject.is_empty() || description.is_empty() {
        return Err(AppError::Validation(
            "Subject and description are required".to_string(),
        ));
    }

    let priority = priority.unwrap_or_else(|| "medium".to_string());
    let valid_priorities = ["low", "medium", "high", "urgent"];
    if !valid_priorities.contains(&priority.as_str()) {
        return Err(AppError::Validation(format!(
            "Invalid priority. Must be one of: {}",
            valid_priorities.join(", ")
        )));
    }

    let ticket_id: String = sqlx::query_scalar(
        "
        INSERT INTO tickets (subject, description, customer_id, department_id, priority)
        VALUES ($1, $2, $3::uuid, $4::uuid, $5::ticket_priority)
        RETURNING CAST(id AS TEXT) as id
        ",
    )
    .bind(&subject)
    .bind(&description)
    .bind(&claims.sub)
    .bind(&department_id)
    .bind(&priority)
    .fetch_one(&state.pool)
    .await?;

    let mut attachments = Vec::new();

    if let (Some(data), Some(name), Some(mime)) = (file_data, filename, mime_type) {
        let file_id = uuid::Uuid::new_v4();
        let ext = name.rsplit('.').next().unwrap_or("bin");
        let stored_filename = format!("{}.{}", file_id, ext);
        let storage_path = format!("tickets/{}/{}", ticket_id, stored_filename);

        let dir = format!("{}/tickets/{}", state.config.upload_dir, ticket_id);
        std::fs::create_dir_all(&dir)
            .map_err(|e| AppError::Storage(format!("Failed to create directory: {}", e)))?;
        std::fs::write(format!("{}/{}", dir, stored_filename), &data)
            .map_err(|e| AppError::Storage(format!("Failed to write file: {}", e)))?;

        let attachment = sqlx::query_as::<_, Attachment>(
            "
            INSERT INTO attachments (filename, original_name, mime_type, file_size, storage_path, attachment_type, ticket_id, uploaded_by)
            VALUES ($1, $2, $3, $4, $5, 'ticket', $6::uuid, $7::uuid)
            RETURNING
                CAST(id AS TEXT) as id,
                filename,
                original_name,
                mime_type,
                file_size,
                CAST(uploaded_by AS TEXT) as uploaded_by,
                CAST(created_at AS TEXT) as created_at
            ",
        )
        .bind(&stored_filename)
        .bind(&name)
        .bind(&mime)
        .bind(data.len() as i64)
        .bind(&storage_path)
        .bind(&ticket_id)
        .bind(&claims.sub)
        .fetch_one(&state.pool)
        .await?;

        attachments.push(attachment);
    }

    let ticket = fetch_ticket(&state.pool, &ticket_id).await?;
    Ok(Json(TicketWithAttachments {
        ticket,
        attachments,
    }))
}

pub async fn update_ticket(
    State(state): State<AppState>,
    axum::Extension(_claims): axum::Extension<Claims>,
    Path(ticket_id): Path<String>,
    Json(payload): Json<UpdateTicketRequest>,
) -> Result<Json<Ticket>, AppError> {
    let valid_statuses = ["open", "in_progress", "waiting", "resolved", "closed"];
    if let Some(ref status) = payload.status {
        if !valid_statuses.contains(&status.as_str()) {
            return Err(AppError::Validation(format!(
                "Invalid status. Must be one of: {}",
                valid_statuses.join(", ")
            )));
        }
    }

    let valid_priorities = ["low", "medium", "high", "urgent"];
    if let Some(ref priority) = payload.priority {
        if !valid_priorities.contains(&priority.as_str()) {
            return Err(AppError::Validation(format!(
                "Invalid priority. Must be one of: {}",
                valid_priorities.join(", ")
            )));
        }
    }

    let result = sqlx::query(
        r#"
        UPDATE tickets
        SET
            status = COALESCE($2::ticket_status, status),
            priority = COALESCE($3::ticket_priority, priority),
            assigned_agent_id = COALESCE($4::uuid, assigned_agent_id),
            department_id = COALESCE($5::uuid, department_id),
            updated_at = NOW()
        WHERE id = $1::uuid
        "#,
    )
    .bind(&ticket_id)
    .bind(&payload.status)
    .bind(&payload.priority)
    .bind(&payload.assigned_agent_id)
    .bind(&payload.department_id)
    .execute(&state.pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    let ticket = fetch_ticket(&state.pool, &ticket_id).await?;
    Ok(Json(ticket))
}

pub async fn upload_attachment(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(ticket_id): Path<String>,
    mut multipart: Multipart,
) -> Result<Json<Attachment>, AppError> {
    let mut file_data: Option<Vec<u8>> = None;
    let mut filename: Option<String> = None;
    let mut mime_type: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?
    {
        let name = field.name().unwrap_or("file").to_string();
        if name == "file" {
            filename = Some(field.file_name().unwrap_or("unknown").to_string());
            mime_type = Some(
                field
                    .content_type()
                    .unwrap_or("application/octet-stream")
                    .to_string(),
            );
            file_data = Some(
                field
                    .bytes()
                    .await
                    .map_err(|e| AppError::BadRequest(e.to_string()))?
                    .to_vec(),
            );
        }
    }

    let file_data = file_data.ok_or(AppError::BadRequest("No file provided".to_string()))?;
    let filename = filename.ok_or(AppError::BadRequest("No filename".to_string()))?;
    let mime_type = mime_type.unwrap_or_else(|| "application/octet-stream".to_string());

    let file_id = uuid::Uuid::new_v4();
    let ext = filename.rsplit('.').next().unwrap_or("bin");
    let stored_filename = format!("{}.{}", file_id, ext);
    let storage_path = format!("tickets/{}/{}", ticket_id, stored_filename);

    let dir = format!("{}/tickets/{}", state.config.upload_dir, ticket_id);
    std::fs::create_dir_all(&dir)
        .map_err(|e| AppError::Storage(format!("Failed to create directory: {}", e)))?;
    std::fs::write(format!("{}/{}", dir, stored_filename), &file_data)
        .map_err(|e| AppError::Storage(format!("Failed to write file: {}", e)))?;

    let attachment = sqlx::query_as::<_, Attachment>(
        r#"
        INSERT INTO attachments (filename, original_name, mime_type, file_size, storage_path, attachment_type, ticket_id, uploaded_by)
        VALUES ($1, $2, $3, $4, $5, 'ticket', $6::uuid, $7::uuid)
        RETURNING
            CAST(id AS TEXT) as "id!",
            filename,
            original_name,
            mime_type,
            file_size,
            CAST(uploaded_by AS TEXT) as "uploaded_by!",
            CAST(created_at AS TEXT) as "created_at!"
        "#,
    )
    .bind(&stored_filename)
    .bind(&filename)
    .bind(&mime_type)
    .bind(file_data.len() as i64)
    .bind(&storage_path)
    .bind(&ticket_id)
    .bind(&claims.sub)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(attachment))
}

pub async fn list_attachments(
    State(state): State<AppState>,
    Path(ticket_id): Path<String>,
) -> Result<Json<Vec<Attachment>>, AppError> {
    let attachments = sqlx::query_as::<_, Attachment>(
        r#"
        SELECT
            CAST(id AS TEXT) as "id!",
            filename,
            original_name,
            mime_type,
            file_size,
            CAST(uploaded_by AS TEXT) as "uploaded_by!",
            CAST(created_at AS TEXT) as "created_at!"
        FROM attachments
        WHERE ticket_id = $1::uuid AND attachment_type = 'ticket'
        ORDER BY created_at DESC
        "#,
    )
    .bind(&ticket_id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(attachments))
}

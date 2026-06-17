use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::{config::Config, notifications::email::send_email};

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailJob {
    pub to: String,
    pub subject: String,
    pub body: String,
}

pub struct EmailJobSender {
    tx: tokio::sync::mpsc::UnboundedSender<EmailJob>,
}

impl EmailJobSender {
    pub fn new(config: Config, pool: PgPool) -> Self {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<EmailJob>();

        let processor = EmailJobProcessor { config, pool, rx };

        tokio::spawn(processor.run());

        Self { tx }
    }

    pub fn send(&self, job: EmailJob) -> Result<(), String> {
        self.tx
            .send(job)
            .map_err(|e| format!("Failed to queue email job: {}", e))
    }
}

struct EmailJobProcessor {
    config: Config,
    pool: PgPool,
    rx: tokio::sync::mpsc::UnboundedReceiver<EmailJob>,
}

#[derive(Debug, sqlx::FromRow)]
struct FailedJobRow {
    id: i64,
    payload: serde_json::Value,
}

impl EmailJobProcessor {
    async fn run(mut self) {
        while let Some(job) = self.rx.recv().await {
            self.process_job(job).await;
        }
    }

    async fn process_job(&self, job: EmailJob) {
        let max_retries = 3;
        let mut attempt = 0;

        loop {
            attempt += 1;

            match send_email(&self.config, &job.to, &job.subject, &job.body).await {
                Ok(_) => {
                    tracing::info!("Email sent successfully to {}", job.to);
                    return;
                }
                Err(e) => {
                    tracing::error!("Email attempt {}/{} failed: {}", attempt, max_retries, e);

                    if attempt >= max_retries {
                        let payload = serde_json::to_value(&job).unwrap();
                        if let Err(db_err) = sqlx::query(
                            r#"
                            INSERT INTO failed_jobs (job_type, payload, error_message, attempts, last_attempt_at)
                            VALUES ('email', $1, $2, $3, NOW())
                            "#,
                        )
                        .bind(payload)
                        .bind(e.to_string())
                        .bind(attempt)
                        .execute(&self.pool)
                        .await
                        {
                            tracing::error!("Failed to persist failed job: {:?}", db_err);
                        }
                        return;
                    }

                    let delay = std::time::Duration::from_secs(1 * 2u64.pow(attempt as u32 - 1));
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }
}

pub async fn retry_failed_jobs(config: &Config, pool: &PgPool) -> Result<u64, sqlx::Error> {
    let failed_jobs = sqlx::query_as::<_, FailedJobRow>(
        r#"
        SELECT id, payload
        FROM failed_jobs
        WHERE resolved_at IS NULL AND attempts < max_attempts
        ORDER BY created_at
        LIMIT 10
        "#,
    )
    .fetch_all(pool)
    .await?;

    let mut retried = 0;

    for job in &failed_jobs {
        let email_job: EmailJob =
            serde_json::from_value(job.payload.clone()).unwrap_or_else(|_| EmailJob {
                to: String::new(),
                subject: String::new(),
                body: String::new(),
            });

        if email_job.to.is_empty() {
            continue;
        }

        match send_email(config, &email_job.to, &email_job.subject, &email_job.body).await {
            Ok(_) => {
                sqlx::query(r#"UPDATE failed_jobs SET resolved_at = NOW() WHERE id = $1"#)
                    .bind(job.id)
                    .execute(pool)
                    .await?;
                retried += 1;
            }
            Err(e) => {
                sqlx::query(
                    r#"
                    UPDATE failed_jobs
                    SET attempts = attempts + 1, error_message = $2, last_attempt_at = NOW()
                    WHERE id = $1
                    "#,
                )
                .bind(job.id)
                .bind(e.to_string())
                .execute(pool)
                .await?;
            }
        }
    }

    Ok(retried)
}

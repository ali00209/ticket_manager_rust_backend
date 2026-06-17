use lettre::{
    message::header::ContentType, transport::smtp::authentication::Credentials, Message,
    AsyncSmtpTransport, AsyncTransport, Tokio1Executor,
};

use crate::config::Config;

/// Send an email using the configured SMTP server.
pub async fn send_email(
    config: &Config,
    to: &str,
    subject: &str,
    html_body: &str,
) -> Result<(), String> {
    let email = Message::builder()
        .from(config.smtp_from.parse().map_err(|e| format!("Invalid from address: {}", e))?)
        .to(to.parse().map_err(|e| format!("Invalid to address: {}", e))?)
        .subject(subject)
        .header(ContentType::TEXT_HTML)
        .body(html_body.to_string())
        .map_err(|e| format!("Failed to build email: {}", e))?;

    let credentials = Credentials::new(config.smtp_user.clone(), config.smtp_pass.clone());

    let transport = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp_host)
        .map_err(|e| format!("Failed to create SMTP transport: {}", e))?
        .port(config.smtp_port)
        .credentials(credentials)
        .build();

    transport
        .send(email)
        .await
        .map_err(|e| format!("Failed to send email: {}", e))?;

    Ok(())
}

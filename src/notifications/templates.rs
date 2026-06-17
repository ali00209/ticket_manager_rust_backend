/// Email template for new ticket notification.
pub fn ticket_created(ticket_subject: &str, ticket_id: &str) -> (String, String) {
    let subject = format!("New Ticket Created: {}", ticket_subject);
    let body = format!(
        r#"
        <html>
        <body>
            <h2>New Ticket Created</h2>
            <p>A new support ticket has been created:</p>
            <p><strong>Subject:</strong> {}</p>
            <p><a href="http://localhost:3000/tickets/{}">View Ticket</a></p>
            <p>Please review and assign this ticket as soon as possible.</p>
        </body>
        </html>
        "#,
        ticket_subject, ticket_id
    );
    (subject, body)
}

/// Email template for ticket status change notification.
pub fn ticket_status_changed(
    ticket_subject: &str,
    ticket_id: &str,
    old_status: &str,
    new_status: &str,
) -> (String, String) {
    let subject = format!("Ticket Status Updated: {}", ticket_subject);
    let body = format!(
        r#"
        <html>
        <body>
            <h2>Ticket Status Updated</h2>
            <p><strong>Ticket:</strong> {}</p>
            <p><strong>Status changed:</strong> {} → {}</p>
            <p><a href="http://localhost:3000/tickets/{}">View Ticket</a></p>
        </body>
        </html>
        "#,
        ticket_subject, old_status, new_status, ticket_id
    );
    (subject, body)
}

/// Email template for ticket assignment notification.
pub fn ticket_assigned(
    ticket_subject: &str,
    ticket_id: &str,
    agent_name: &str,
) -> (String, String) {
    let subject = format!("Ticket Assigned to You: {}", ticket_subject);
    let body = format!(
        r#"
        <html>
        <body>
            <h2>Ticket Assigned</h2>
            <p>Hello {},</p>
            <p>A ticket has been assigned to you:</p>
            <p><strong>Subject:</strong> {}</p>
            <p><a href="http://localhost:3000/tickets/{}">View Ticket</a></p>
            <p>Please review and respond promptly.</p>
        </body>
        </html>
        "#,
        agent_name, ticket_subject, ticket_id
    );
    (subject, body)
}

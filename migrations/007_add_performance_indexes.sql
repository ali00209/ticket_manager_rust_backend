-- Performance optimization indexes
-- Composite indexes for filtered/sorted queries

-- Tickets: customer listing (customer_id, created_at DESC)
CREATE INDEX idx_tickets_customer_created ON tickets(customer_id, created_at DESC);

-- Tickets: agent listing via department join (department_id, created_at DESC)
CREATE INDEX idx_tickets_department_created ON tickets(department_id, created_at DESC);

-- Tickets: filtered queries with status/priority
CREATE INDEX idx_tickets_status_created ON tickets(status, created_at DESC);
CREATE INDEX idx_tickets_priority_created ON tickets(priority, created_at DESC);

-- Users: listing by role with sort
CREATE INDEX idx_users_role_created ON users(role, created_at DESC);

-- Users: department agent lookup (department_id, role, is_active)
CREATE INDEX idx_users_dept_role_active ON users(department_id, role) WHERE is_active = true;

-- Messages: sender name lookup for chat
CREATE INDEX idx_messages_sender_id ON messages(sender_id) INCLUDE (ticket_id);

-- Attachments: filtered by ticket and type
CREATE INDEX idx_attachments_ticket_type ON attachments(ticket_id, attachment_type);

-- Failed jobs: retry query optimization
CREATE INDEX idx_failed_jobs_retry ON failed_jobs(created_at) WHERE resolved_at IS NULL AND attempts < max_attempts;

-- Create tickets table
CREATE TYPE ticket_status AS ENUM ('open', 'in_progress', 'waiting', 'resolved', 'closed');
CREATE TYPE ticket_priority AS ENUM ('low', 'medium', 'high', 'urgent');

CREATE TABLE tickets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subject VARCHAR(500) NOT NULL,
    description TEXT NOT NULL,
    status ticket_status NOT NULL DEFAULT 'open',
    priority ticket_priority NOT NULL DEFAULT 'medium',
    customer_id UUID NOT NULL,
    department_id UUID NOT NULL,
    assigned_agent_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT fk_tickets_customer
        FOREIGN KEY (customer_id)
        REFERENCES users(id)
        ON DELETE CASCADE,

    CONSTRAINT fk_tickets_department
        FOREIGN KEY (department_id)
        REFERENCES departments(id)
        ON DELETE RESTRICT,

    CONSTRAINT fk_tickets_agent
        FOREIGN KEY (assigned_agent_id)
        REFERENCES users(id)
        ON DELETE SET NULL
);

CREATE INDEX idx_tickets_status ON tickets(status);
CREATE INDEX idx_tickets_priority ON tickets(priority);
CREATE INDEX idx_tickets_customer ON tickets(customer_id);
CREATE INDEX idx_tickets_department ON tickets(department_id);
CREATE INDEX idx_tickets_agent ON tickets(assigned_agent_id);
CREATE INDEX idx_tickets_created ON tickets(created_at DESC);

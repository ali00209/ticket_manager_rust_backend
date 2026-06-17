-- Create attachments table
CREATE TYPE attachment_type AS ENUM ('ticket', 'message');

CREATE TABLE attachments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    filename VARCHAR(255) NOT NULL,
    original_name VARCHAR(255) NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    file_size BIGINT NOT NULL,
    storage_path VARCHAR(500) NOT NULL,
    attachment_type attachment_type NOT NULL,
    ticket_id UUID,
    message_id UUID,
    uploaded_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT fk_attachments_ticket
        FOREIGN KEY (ticket_id)
        REFERENCES tickets(id)
        ON DELETE CASCADE,

    CONSTRAINT fk_attachments_message
        FOREIGN KEY (message_id)
        REFERENCES messages(id)
        ON DELETE CASCADE,

    CONSTRAINT fk_attachments_uploader
        FOREIGN KEY (uploaded_by)
        REFERENCES users(id)
        ON DELETE CASCADE,

    CONSTRAINT chk_attachment_parent
        CHECK (
            (attachment_type = 'ticket' AND ticket_id IS NOT NULL AND message_id IS NULL)
            OR
            (attachment_type = 'message' AND message_id IS NOT NULL AND ticket_id IS NULL)
        )
);

CREATE INDEX idx_attachments_ticket ON attachments(ticket_id);
CREATE INDEX idx_attachments_message ON attachments(message_id);

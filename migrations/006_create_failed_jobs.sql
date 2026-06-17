-- Create failed_jobs table for email retry
CREATE TABLE failed_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_type VARCHAR(100) NOT NULL,
    payload JSONB NOT NULL,
    error_message TEXT,
    attempts INT NOT NULL DEFAULT 0,
    max_attempts INT NOT NULL DEFAULT 3,
    last_attempt_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at TIMESTAMPTZ
);

CREATE INDEX idx_failed_jobs_type ON failed_jobs(job_type);
CREATE INDEX idx_failed_jobs_unresolved ON failed_jobs(resolved_at) WHERE resolved_at IS NULL;

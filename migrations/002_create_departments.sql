-- Create departments table
CREATE TABLE departments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add foreign key from users to departments
ALTER TABLE users
    ADD CONSTRAINT fk_users_department
    FOREIGN KEY (department_id)
    REFERENCES departments(id)
    ON DELETE SET NULL;

CREATE INDEX idx_departments_name ON departments(name);

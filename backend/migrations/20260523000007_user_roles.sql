-- Migration: Create user_roles table for admin management
CREATE TABLE IF NOT EXISTS user_roles (
    user_id UUID,
    email VARCHAR(255) NOT NULL,
    role VARCHAR(50) NOT NULL DEFAULT 'admin',
    granted_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (email)
);

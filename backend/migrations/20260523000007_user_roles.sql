-- Migration: Create user_roles table for admin management
CREATE TABLE IF NOT EXISTS user_roles (
    user_id UUID,
    email VARCHAR(255) NOT NULL,
    role VARCHAR(50) NOT NULL DEFAULT 'admin',
    granted_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (email)
);

-- Activation de la Row Level Security (RLS)
ALTER TABLE user_roles ENABLE ROW LEVEL SECURITY;

-- Blocage de tout accès public REST direct via Supabase (PostgREST)
-- Notre serveur Rust (connecté en superutilisateur postgres) outrepasse RLS et manipule cette table directement.
CREATE POLICY "Deny all public access" ON user_roles FOR ALL TO public USING (false);

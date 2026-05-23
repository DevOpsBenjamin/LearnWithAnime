-- Migration: Enable RLS and deny public access on user_roles table
ALTER TABLE user_roles ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Deny all public access" ON user_roles FOR ALL TO public USING (false);

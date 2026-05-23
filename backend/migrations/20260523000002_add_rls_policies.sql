-- Crée des politiques explicites de refus d'accès pour toutes les tables.
-- Cela bloque tout accès via l'API REST publique de Supabase (PostgREST),
-- tout en permettant à notre backend Rust (connecté en superutilisateur postgres) d'opérer librement.

CREATE POLICY "Deny all public access" ON users FOR ALL TO public USING (false);
CREATE POLICY "Deny all public access" ON decks FOR ALL TO public USING (false);
CREATE POLICY "Deny all public access" ON cards FOR ALL TO public USING (false);
CREATE POLICY "Deny all public access" ON user_card_progress FOR ALL TO public USING (false);
CREATE POLICY "Deny all public access" ON _sqlx_migrations FOR ALL TO public USING (false);

-- 1. Active la Row Level Security (RLS) sur toutes nos tables
ALTER TABLE users ENABLE ROW LEVEL SECURITY;
ALTER TABLE decks ENABLE ROW LEVEL SECURITY;
ALTER TABLE cards ENABLE ROW LEVEL SECURITY;
ALTER TABLE user_card_progress ENABLE ROW LEVEL SECURITY;

-- 2. Optionnel : Active la RLS sur la table de migrations internes SQLx pour le linter Supabase
ALTER TABLE _sqlx_migrations ENABLE ROW LEVEL SECURITY;

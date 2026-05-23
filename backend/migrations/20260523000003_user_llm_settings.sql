-- Migration pour stocker les paramètres d'IA par utilisateur en base de données

CREATE TABLE IF NOT EXISTS user_llm_settings (
    user_id UUID PRIMARY KEY,
    api_url VARCHAR(255) NOT NULL DEFAULT 'http://localhost:1337/v1',
    api_key TEXT,
    model VARCHAR(100) NOT NULL DEFAULT 'minimax-2.7b',
    temperature REAL NOT NULL DEFAULT 0.2,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Activation de la Row Level Security (RLS)
ALTER TABLE user_llm_settings ENABLE ROW LEVEL SECURITY;

-- Blocage de tout accès public REST direct via Supabase (PostgREST)
-- Notre serveur Rust (connecté en superutilisateur postgres) outrepasse RLS et manipule cette table directement.
CREATE POLICY "Deny all public access" ON user_llm_settings FOR ALL TO public USING (false);

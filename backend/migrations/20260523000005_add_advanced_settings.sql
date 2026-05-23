-- Migration pour ajouter les paramètres d'IA avancés (Top P, Max tokens, Frequency penalty)

ALTER TABLE user_llm_settings 
ADD COLUMN IF NOT EXISTS top_p REAL NOT NULL DEFAULT 1.0,
ADD COLUMN IF NOT EXISTS frequency_penalty REAL NOT NULL DEFAULT 0.0,
ADD COLUMN IF NOT EXISTS max_tokens INT NOT NULL DEFAULT 2048;

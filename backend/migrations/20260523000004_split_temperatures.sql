-- Migration pour séparer la température globale en deux paramètres distincts : évaluation et indices

ALTER TABLE user_llm_settings 
ADD COLUMN IF NOT EXISTS temperature_eval REAL NOT NULL DEFAULT 0.1,
ADD COLUMN IF NOT EXISTS temperature_hint REAL NOT NULL DEFAULT 0.7;

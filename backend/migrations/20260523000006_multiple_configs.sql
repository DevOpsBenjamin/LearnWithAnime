-- Migration pour supporter plusieurs configurations d'IA par utilisateur

-- 1. Supprime la contrainte de clé primaire existante sur user_id
ALTER TABLE user_llm_settings DROP CONSTRAINT IF EXISTS user_llm_settings_pkey;

-- 2. Ajoute la colonne config_name avec une valeur par défaut
ALTER TABLE user_llm_settings ADD COLUMN IF NOT EXISTS config_name VARCHAR(100) NOT NULL DEFAULT 'Défaut';

-- 3. Ajoute la colonne is_active pour suivre la configuration active
ALTER TABLE user_llm_settings ADD COLUMN IF NOT EXISTS is_active BOOLEAN NOT NULL DEFAULT true;

-- 4. Établit la nouvelle clé primaire composite sur (user_id, config_name)
ALTER TABLE user_llm_settings ADD PRIMARY KEY (user_id, config_name);

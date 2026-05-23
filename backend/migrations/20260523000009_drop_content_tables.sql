-- Drop content tables (replaced by JSON catalog in data/)
-- Order matters due to foreign key constraints
DROP TABLE IF EXISTS user_card_progress;
DROP TABLE IF EXISTS cards;
DROP TABLE IF EXISTS decks;

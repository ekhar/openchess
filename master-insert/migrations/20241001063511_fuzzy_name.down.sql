-- Add down migration script here
DROP INDEX IF EXISTS idx_white_player_trgm;
DROP INDEX IF EXISTS idx_black_player_trgm;

DROP EXTENSION IF EXISTS pg_trgm;

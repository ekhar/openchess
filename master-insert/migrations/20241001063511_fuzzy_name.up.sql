-- Add up migration script here
CREATE EXTENSION IF NOT EXISTS pg_trgm;
CREATE INDEX idx_white_player_trgm ON games USING GIN (white_player gin_trgm_ops);
CREATE INDEX idx_black_player_trgm ON games USING GIN (black_player gin_trgm_ops);

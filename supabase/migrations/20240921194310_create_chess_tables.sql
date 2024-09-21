-- Enable UUID extension (if not already enabled)
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Create enum for game results
CREATE TYPE game_result AS ENUM ('white_win', 'black_win', 'draw');

-- Create positions table
CREATE TABLE positions (
    compressed_fen BYTEA PRIMARY KEY
);

-- Create position_stats table
CREATE TABLE position_stats (
    compressed_fen BYTEA PRIMARY KEY,
    total_games INTEGER DEFAULT 0,
    master_games INTEGER DEFAULT 0,
    player_games INTEGER DEFAULT 0,
    white_wins INTEGER DEFAULT 0,
    black_wins INTEGER DEFAULT 0,
    draws INTEGER DEFAULT 0,
    FOREIGN KEY (compressed_fen) REFERENCES positions(compressed_fen)
);

-- Create master_games table
CREATE TABLE master_games (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    white_player TEXT NOT NULL,
    black_player TEXT NOT NULL,
    date DATE,
    result game_result NOT NULL,
    compressed_pgn BYTEA NOT NULL,
    position_sequence BYTEA[] NOT NULL,
    event TEXT,
    site TEXT,
    white_elo INTEGER,
    black_elo INTEGER
);

-- Create player_games table
CREATE TABLE player_games (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    white_player TEXT NOT NULL,
    black_player TEXT NOT NULL,
    date DATE,
    result game_result NOT NULL,
    compressed_pgn BYTEA NOT NULL,
    position_sequence BYTEA[] NOT NULL,
    event TEXT,
    site TEXT,
    white_elo INTEGER,
    black_elo INTEGER
);

-- Create indexes
CREATE INDEX idx_master_games_white_player ON master_games(white_player);
CREATE INDEX idx_master_games_black_player ON master_games(black_player);
CREATE INDEX idx_player_games_white_player ON player_games(white_player);
CREATE INDEX idx_player_games_black_player ON player_games(black_player);
CREATE INDEX idx_master_games_position_sequence ON master_games USING GIN(position_sequence);
CREATE INDEX idx_player_games_position_sequence ON player_games USING GIN(position_sequence);

-- Function to update or insert position stats
CREATE OR REPLACE FUNCTION upsert_position_stats(compressed_fen_param BYTEA, game_type TEXT, result_param game_result)
RETURNS VOID AS $$
BEGIN
    -- Ensure the position exists in the positions table
    INSERT INTO positions (compressed_fen)
    VALUES (compressed_fen_param)
    ON CONFLICT (compressed_fen) DO NOTHING;

    -- Update or insert stats
    INSERT INTO position_stats (compressed_fen, total_games, master_games, player_games, white_wins, black_wins, draws)
    VALUES (
        compressed_fen_param, 
        1, 
        (game_type = 'master')::INT, 
        (game_type = 'player')::INT,
        (result_param = 'white_win')::INT,
        (result_param = 'black_win')::INT,
        (result_param = 'draw')::INT
    )
    ON CONFLICT (compressed_fen) DO UPDATE
    SET total_games = position_stats.total_games + 1,
        master_games = position_stats.master_games + (game_type = 'master')::INT,
        player_games = position_stats.player_games + (game_type = 'player')::INT,
        white_wins = position_stats.white_wins + (result_param = 'white_win')::INT,
        black_wins = position_stats.black_wins + (result_param = 'black_win')::INT,
        draws = position_stats.draws + (result_param = 'draw')::INT;
END;
$$ LANGUAGE plpgsql;

-- Function to update position stats for a game
CREATE OR REPLACE FUNCTION update_game_position_stats(game_type TEXT, result_param game_result, position_sequence BYTEA[])
RETURNS VOID AS $$
DECLARE
    compressed_fen BYTEA;
BEGIN
    FOREACH compressed_fen IN ARRAY position_sequence
    LOOP
        PERFORM upsert_position_stats(compressed_fen, game_type, result_param);
    END LOOP;
END;
$$ LANGUAGE plpgsql;

-- Trigger function for master games
CREATE OR REPLACE FUNCTION update_master_game_stats()
RETURNS TRIGGER AS $$
BEGIN
    PERFORM update_game_position_stats('master', NEW.result, NEW.position_sequence);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger function for player games
CREATE OR REPLACE FUNCTION update_player_game_stats()
RETURNS TRIGGER AS $$
BEGIN
    PERFORM update_game_position_stats('player', NEW.result, NEW.position_sequence);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create triggers
CREATE TRIGGER master_game_insert_trigger
AFTER INSERT ON master_games
FOR EACH ROW
EXECUTE FUNCTION update_master_game_stats();

CREATE TRIGGER player_game_insert_trigger
AFTER INSERT ON player_games
FOR EACH ROW
EXECUTE FUNCTION update_player_game_stats();

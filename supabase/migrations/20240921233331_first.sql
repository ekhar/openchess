CREATE EXTENSION IF NOT EXISTS pg_trgm;
CREATE TYPE game_result AS ENUM ('white', 'black', 'draw');
CREATE TYPE site AS ENUM ('chesscom', 'lichess', 'custom');
CREATE TYPE speed AS ENUM (
    'ultrabullet',
    'bullet',
    'blitz',
    'rapid',
    'classical',
    'correspondence'
);

-- Create positions table
CREATE TABLE positions (
    id SERIAL PRIMARY KEY,
    compressed_fen BYTEA UNIQUE
);

-- Master Games Table
CREATE TABLE master_games (
    id SERIAL PRIMARY KEY,
    eco VARCHAR(4) NOT NULL,
    white_player TEXT NOT NULL,
    black_player TEXT NOT NULL,
    date DATE,
    result game_result NOT NULL,
    compressed_pgn BYTEA NOT NULL,
    white_elo SMALLINT NOT NULL,
    black_elo SMALLINT NOT NULL,
    time_control speed 
);

CREATE TABLE master_game_positions (
    game_id INTEGER NOT NULL REFERENCES master_games(id),
    position_id INTEGER NOT NULL REFERENCES positions(id),
    move_number INTEGER NOT NULL,
    PRIMARY KEY (game_id, move_number)
);
CREATE INDEX idx_master_game_positions_position ON master_game_positions (position_id);
CREATE INDEX idx_master_game_positions_game ON master_game_positions (game_id);

-- Player Games Table
CREATE TABLE player_games (
    id SERIAL PRIMARY KEY,
    eco VARCHAR(4) NOT NULL,
    white_player TEXT NOT NULL,
    black_player TEXT NOT NULL,
    date DATE,
    result game_result NOT NULL,
    compressed_pgn BYTEA NOT NULL,
    site site,
    white_elo SMALLINT NOT NULL,
    black_elo SMALLINT NOT NULL,
    time_control speed 
);

CREATE TABLE player_game_positions (
    game_id INTEGER NOT NULL REFERENCES player_games(id),
    position_id INTEGER NOT NULL REFERENCES positions(id),
    move_number INTEGER NOT NULL,
    PRIMARY KEY (game_id, move_number)
);

CREATE INDEX idx_player_game_positions_position ON player_game_positions (position_id);
CREATE INDEX idx_player_game_positions_game ON player_game_positions (game_id);


--FEN find exact and fuzzy matches
CREATE INDEX idx_positions_hash ON positions (compressed_fen);

--MASTER
CREATE INDEX idx_master_games_white_player ON master_games USING btree (white_player);
CREATE INDEX idx_master_games_black_player ON master_games USING btree (black_player);
CREATE INDEX idx_master_games_result ON master_games USING btree (result);
CREATE INDEX idx_master_games_eco ON master_games USING btree (eco);
CREATE INDEX idx_master_games_white_elo ON master_games USING btree (white_elo);
CREATE INDEX idx_master_games_black_elo ON master_games USING btree (black_elo);
CREATE INDEX idx_master_games_time_control ON master_games USING btree (time_control);

--PLAYER
CREATE INDEX idx_player_games_white_player ON player_games USING btree (white_player);
CREATE INDEX idx_player_games_black_player ON player_games USING btree (black_player);

CREATE INDEX idx_player_games_white_result ON player_games USING btree (white_player, result);
CREATE INDEX idx_player_games_black_result ON player_games USING btree (black_player, result);

CREATE INDEX idx_player_games_white_white_elo ON player_games USING btree (white_player, white_elo);
CREATE INDEX idx_player_games_black_black_elo ON player_games USING btree (black_player, black_elo);

CREATE INDEX idx_player_games_white_time_control ON player_games USING btree (white_player, time_control);
CREATE INDEX idx_player_games_black_time_control ON player_games USING btree (black_player, time_control);

CREATE INDEX idx_player_games_white_site ON player_games USING btree (white_player, site);
CREATE INDEX idx_player_games_black_site ON player_games USING btree (black_player, site);


CREATE INDEX idx_player_games_white_eco ON player_games USING btree (white_player, eco);
CREATE INDEX idx_player_games_black_eco ON player_games USING btree (black_player, eco);

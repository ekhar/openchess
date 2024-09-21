-- Create enum for game results
CREATE TYPE game_result AS ENUM ('white', 'black', 'draw');
CREATE TYPE site AS ENUM ('chesscom', 'lichess', 'custom');
CREATE TYPE speed AS ENUM (
    'ultraBullet',
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
    eco VARCHAR(3) NOT NULL,
    white_player TEXT NOT NULL,
    black_player TEXT NOT NULL,
    date DATE,
    result game_result NOT NULL,
    compressed_pgn BYTEA NOT NULL,
    position_sequence INTEGER[] NOT NULL, -- Changed from BYTEA[]
    white_elo SMALLINT NOT NULL,
    black_elo SMALLINT NOT NULL,
    time_control speed 
);

-- Player Games Table
CREATE TABLE player_games (
    id SERIAL PRIMARY KEY,
    eco VARCHAR(3) NOT NULL,
    white_player TEXT NOT NULL,
    black_player TEXT NOT NULL,
    date DATE,
    result game_result NOT NULL,
    compressed_pgn BYTEA NOT NULL,
    position_sequence INTEGER[] NOT NULL, -- Changed from BYTEA[]
    site site,
    white_elo SMALLINT NOT NULL,
    black_elo SMALLINT NOT NULL,
    time_control speed 
);

-- Example for master_games
ALTER TABLE master_games
ADD CONSTRAINT fk_master_positions
FOREIGN KEY (position_sequence)
REFERENCES positions(id);

-- Example for player_games
ALTER TABLE player_games
ADD CONSTRAINT fk_player_positions
FOREIGN KEY (position_sequence)
REFERENCES positions(id);

-- FEN
CREATE INDEX idx_master_games_positions ON master_games USING GIN (position_sequence);
CREATE INDEX idx_player_games_positions ON player_games USING GIN (position_sequence);

CREATE INDEX idx_positions_hash ON positions USING hash (compressed_fen);
CREATE INDEX idx_positions_gib ON positions USING GIN (compressed_fen);

--MASTER
CREATE INDEX idx_master_games_white_elo_date ON master_games USING btree (date);
CREATE INDEX idx_master_games_white_player ON master_games USING btree (white_player);
CREATE INDEX idx_master_games_black_player ON master_games USING btree (black_player);
CREATE INDEX idx_master_games_date ON master_games USING btree (date);
CREATE INDEX idx_master_pgn ON master_games USING gin(compressed_pgn);
CREATE INDEX idx_master_games_result ON master_games USING hash (result);
CREATE INDEX idx_master_games_eco ON master_games USING btree (eco);
CREATE INDEX idx_master_games_white_elo ON master_games USING btree (white_elo);
CREATE INDEX idx_master_games_black_elo ON master_games USING btree (black_elo);
CREATE INDEX idx_master_games_time_control ON master_games USING hash (time_control);

--PLAYER
CREATE INDEX idx_player_pgn ON master_games USING gin(compressed_pgn);

CREATE INDEX idx_player_games_white_player ON player_games USING btree (white_player);
CREATE INDEX idx_player_games_black_player ON player_games USING btree (black_player);

CREATE INDEX idx_player_games_white_date ON player_games USING btree (white_player, date);
CREATE INDEX idx_player_games_black_date ON player_games USING btree (black_player, date);

CREATE INDEX idx_player_games_white_result ON player_games USING btree (white_player, result);
CREATE INDEX idx_player_games_black_result ON player_games USING btree (black_player, result);

CREATE INDEX idx_player_games_white_white_elo ON player_games USING btree (white_player, white_elo);
CREATE INDEX idx_player_games_black_black_elo ON player_games USING btree (black_player, black_elo);

CREATE INDEX idx_player_games_white_time_control ON player_games USING btree (white_player, time_control);
CREATE INDEX idx_player_games_black_time_control ON player_games USING btree (black_player, time_control);

CREATE INDEX idx_player_games_white_site ON player_games USING btree (white_player, site);
CREATE INDEX idx_player_games_black_site ON player_games USING btree (black_player, site);


CREATE INDEX idx_player_games_white_site ON player_games USING btree (white_player, eco);
CREATE INDEX idx_player_games_black_site ON player_games USING btree (black_player, eco);

-- === Up Migrations ===

-- Create initial tables and types
CREATE TYPE result AS ENUM (
    'white',
    'black',
    'draw'
);

CREATE TYPE chess_speed AS ENUM (
    'UltraBullet',
    'Bullet',
    'Blitz',
    'Rapid',
    'Classical',
    'Correspondence'
);

CREATE TABLE games (
    id SERIAL PRIMARY KEY,
    eco VARCHAR,
    white_player VARCHAR NOT NULL,
    black_player VARCHAR NOT NULL,
    date DATE,
    result result NOT NULL,
    white_elo INTEGER NOT NULL,
    black_elo INTEGER NOT NULL,
    time_control chess_speed,
    pgn_moves BYTEA NOT NULL
);

CREATE TABLE positions (
    id SERIAL PRIMARY KEY,
    game_id INTEGER NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    move_number SMALLINT NOT NULL,
    position BYTEA NOT NULL CHECK (octet_length(position) = 32)
);

-- Partition positions table
CREATE TABLE positions_new (
    id SERIAL NOT NULL,
    game_id INTEGER NOT NULL,
    move_number SMALLINT NOT NULL,
    position BYTEA NOT NULL CHECK (octet_length(position) = 32),
    PRIMARY KEY (game_id, id)
) PARTITION BY RANGE (game_id);

-- Define partition ranges
DO $$
DECLARE
    start_id INTEGER := 1;
    end_id INTEGER := 1000000;
    step INTEGER := 1000000;
    partition_name TEXT;
BEGIN
    FOR i IN 0..13 LOOP
        partition_name := format('positions_p%s_%s', start_id, end_id);
        EXECUTE format('
            CREATE TABLE %I PARTITION OF positions_new
            FOR VALUES FROM (%L) TO (%L);
        ', partition_name, start_id, end_id + 1);
        start_id := start_id + step;
        end_id := end_id + step;
    END LOOP;

    EXECUTE format('
        CREATE TABLE %I PARTITION OF positions_new
        FOR VALUES FROM (%L) TO (MAXVALUE);
    ', 'positions_p_remaining', start_id);
END $$;

-- Migrate data to the new partitioned table
INSERT INTO positions_new (id, game_id, move_number, position)
SELECT id, game_id, move_number, position
FROM positions;

-- Rename tables to finalize partitioning
ALTER TABLE positions RENAME TO positions_old;
ALTER TABLE positions_new RENAME TO positions;

-- Drop the old positions table
DROP TABLE positions_old;

-- Create indexes on positions partitions
DO $$
DECLARE
    partition RECORD;
BEGIN
    FOR partition IN
        SELECT tablename
        FROM pg_tables
        WHERE schemaname = 'public' AND tablename LIKE 'positions_p%'
    LOOP
        EXECUTE format('CREATE INDEX IF NOT EXISTS idx_%I_position ON %I(position);', partition.tablename, partition.tablename);
        EXECUTE format('CREATE INDEX IF NOT EXISTS idx_%I_game_id ON %I(game_id);', partition.tablename, partition.tablename);
    END LOOP;
END $$;

-- Create indexes on the games table
CREATE INDEX IF NOT EXISTS idx_games_result ON games(result);
CREATE INDEX IF NOT EXISTS idx_games_time_control ON games(time_control);
CREATE INDEX IF NOT EXISTS idx_games_eco ON games(eco);
CREATE INDEX IF NOT EXISTS idx_games_white_player ON games(white_player);
CREATE INDEX IF NOT EXISTS idx_games_black_player ON games(black_player);
CREATE INDEX IF NOT EXISTS idx_games_time_control_eco ON games(time_control, eco);
CREATE INDEX IF NOT EXISTS idx_games_white_black_player ON games(white_player, black_player);
CREATE INDEX IF NOT EXISTS idx_games_result_time_control ON games(result, time_control);

-- Create trigram indexes
CREATE EXTENSION IF NOT EXISTS pg_trgm;
CREATE INDEX idx_white_player_trgm ON games USING GIN (white_player gin_trgm_ops);
CREATE INDEX idx_black_player_trgm ON games USING GIN (black_player gin_trgm_ops);

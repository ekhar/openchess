#!/bin/bash
set -e # Exit on error

# Configuration
DB_NAME="chess_database"
DB_USER="admin"
DB_PASSWORD="9789"
DB_HOST="chess-db"
DB_PORT="5432"
OUTPUT_FILE="scripts/seed.sql"
GAMES_LIMIT=500

echo "Creating chess database seed file with $GAMES_LIMIT games..."

# Create temporary directory for SQL parts
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT # Clean up temp dir on exit

# 1. Create schema section
cat >"$TMP_DIR/01_schema.sql" <<'EOF'
-- Chess Database Seed File
-- Contains structure and data for development environment

-- Set environment
SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

-- Drop existing tables if they exist
DROP TABLE IF EXISTS game_positions CASCADE;
DROP TABLE IF EXISTS positions CASCADE; 
DROP TABLE IF EXISTS unique_positions CASCADE;
DROP TABLE IF EXISTS games CASCADE;
DROP TABLE IF EXISTS migration_progress CASCADE;
DROP TABLE IF EXISTS _sqlx_migrations CASCADE;

-- Drop types
DROP TYPE IF EXISTS public.chess_speed CASCADE;
DROP TYPE IF EXISTS public.result CASCADE;

-- Extensions
CREATE EXTENSION IF NOT EXISTS pg_trgm WITH SCHEMA public;
COMMENT ON EXTENSION pg_trgm IS 'text similarity measurement and index searching based on trigrams';

-- Create custom types
CREATE TYPE public.chess_speed AS ENUM (
    'UltraBullet',
    'Bullet',
    'Blitz',
    'Rapid',
    'Classical',
    'Correspondence'
);

CREATE TYPE public.result AS ENUM (
    'white',
    'black',
    'draw'
);

-- Create tables (simplified without partitioning for development)
CREATE TABLE public.games (
    id integer NOT NULL,
    eco character varying,
    white_player character varying NOT NULL,
    black_player character varying NOT NULL,
    date date,
    result public.result NOT NULL,
    white_elo integer NOT NULL,
    black_elo integer NOT NULL,
    time_control public.chess_speed,
    pgn_moves bytea NOT NULL,
    PRIMARY KEY (id)
);

CREATE SEQUENCE public.games_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.games_id_seq OWNED BY public.games.id;
ALTER TABLE ONLY public.games ALTER COLUMN id SET DEFAULT nextval('public.games_id_seq'::regclass);

CREATE TABLE public.positions (
    id integer NOT NULL,
    game_id integer NOT NULL,
    move_number smallint NOT NULL,
    "position" bytea NOT NULL,
    CONSTRAINT positions_new_position_check CHECK ((octet_length("position") = 32)),
    PRIMARY KEY (id)
);

CREATE SEQUENCE public.positions_new_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.positions_new_id_seq OWNED BY public.positions.id;
ALTER TABLE ONLY public.positions ALTER COLUMN id SET DEFAULT nextval('public.positions_new_id_seq'::regclass);

CREATE TABLE public.unique_positions (
    position_id integer NOT NULL,
    "position" bytea NOT NULL,
    frequency integer DEFAULT 0,
    PRIMARY KEY (position_id)
);

CREATE SEQUENCE public.unique_positions_position_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.unique_positions_position_id_seq OWNED BY public.unique_positions.position_id;
ALTER TABLE ONLY public.unique_positions ALTER COLUMN position_id SET DEFAULT nextval('public.unique_positions_position_id_seq'::regclass);

CREATE UNIQUE INDEX uq_unique_positions_position ON public.unique_positions USING btree ("position");

CREATE TABLE public.game_positions (
    game_id integer NOT NULL,
    position_id integer NOT NULL,
    move_number smallint NOT NULL,
    PRIMARY KEY (game_id, move_number)
);

CREATE TABLE public._sqlx_migrations (
    version bigint NOT NULL,
    description text NOT NULL,
    installed_on timestamp with time zone DEFAULT now() NOT NULL,
    success boolean NOT NULL,
    checksum bytea NOT NULL,
    execution_time bigint NOT NULL,
    PRIMARY KEY (version)
);

CREATE TABLE public.migration_progress (
    step text NOT NULL,
    started timestamp without time zone,
    completed timestamp without time zone,
    rows_processed bigint DEFAULT 0,
    PRIMARY KEY (step)
);

-- Create stored procedures
CREATE PROCEDURE public.link_positions_batch(IN min_id_param bigint, IN max_id_param bigint)
    LANGUAGE plpgsql
    AS $$
DECLARE
    rows_linked BIGINT;
BEGIN
    RAISE NOTICE 'Linking positions from ID % to %', min_id_param, max_id_param;
    
    -- Link positions directly
    INSERT INTO game_positions (game_id, position_id, move_number)
    SELECT p.game_id, up.position_id, p.move_number
    FROM positions p
    JOIN unique_positions up ON p.position = up.position
    WHERE p.id BETWEEN min_id_param AND max_id_param;
    
    GET DIAGNOSTICS rows_linked = ROW_COUNT;
    
    RAISE NOTICE 'Linked % positions from IDs % to %', 
        rows_linked, min_id_param, max_id_param;
END;
$$;

CREATE PROCEDURE public.process_positions_batch(IN min_id_param bigint, IN max_id_param bigint)
    LANGUAGE plpgsql
    AS $$
DECLARE
    rows_inserted BIGINT;
BEGIN
    RAISE NOTICE 'Processing positions from ID % to %', min_id_param, max_id_param;
    
    -- Find unique positions in range and merge with existing
    WITH batch_positions AS (
        SELECT position, COUNT(*) AS frequency
        FROM positions
        WHERE id BETWEEN min_id_param AND max_id_param
        GROUP BY position
    )
    INSERT INTO unique_positions (position, frequency)
    SELECT position, frequency
    FROM batch_positions
    ON CONFLICT (position) DO UPDATE 
    SET frequency = unique_positions.frequency + EXCLUDED.frequency;
    
    GET DIAGNOSTICS rows_inserted = ROW_COUNT;
    
    RAISE NOTICE 'Processed % unique positions from IDs % to %', 
        rows_inserted, min_id_param, max_id_param;
END;
$$;

-- Create indexes
CREATE INDEX idx_games_white_player ON public.games USING btree (white_player);
CREATE INDEX idx_games_black_player ON public.games USING btree (black_player);
CREATE INDEX idx_games_eco ON public.games USING btree (eco);
CREATE INDEX idx_games_result ON public.games USING btree (result);
CREATE INDEX idx_games_time_control ON public.games USING btree (time_control);
CREATE INDEX idx_games_result_time_control ON public.games USING btree (result, time_control);
CREATE INDEX idx_games_time_control_eco ON public.games USING btree (time_control, eco);
CREATE INDEX idx_games_white_black_player ON public.games USING btree (white_player, black_player);
CREATE INDEX idx_white_player_trgm ON public.games USING gin (white_player public.gin_trgm_ops);
CREATE INDEX idx_black_player_trgm ON public.games USING gin (black_player public.gin_trgm_ops);

CREATE INDEX idx_positions_game_id ON public.positions USING btree (game_id);
CREATE INDEX idx_positions_position ON public.positions USING btree ("position");

CREATE INDEX idx_game_positions_position_id ON public.game_positions USING btree (position_id);

CREATE INDEX idx_unique_positions_frequency ON public.unique_positions USING btree (frequency DESC);
CREATE INDEX idx_unique_positions_hash ON public.unique_positions USING hash ("position");
EOF

echo "Schema created."

# 2. Extract games data
echo "Extracting games data..."
docker exec -i $DB_HOST psql -U $DB_USER -d $DB_NAME -t -c "
SELECT format('INSERT INTO public.games (id, eco, white_player, black_player, date, result, white_elo, black_elo, time_control, pgn_moves) VALUES (%s, %L, %L, %L, %L, %L, %s, %s, %L, decode(''%s'', ''hex''));',
  id,
  eco,
  white_player,
  black_player,
  date,
  result,
  white_elo,
  black_elo,
  time_control,
  encode(pgn_moves, 'hex')
)
FROM games
WHERE id < $GAMES_LIMIT
ORDER BY id;" >"$TMP_DIR/02_games_inserts.sql"

GAMES_COUNT=$(grep -c "INSERT INTO" "$TMP_DIR/02_games_inserts.sql" || echo 0)
echo "Extracted $GAMES_COUNT games."

# 3. Extract positions data
echo "Extracting positions data..."
docker exec -i $DB_HOST psql -U $DB_USER -d $DB_NAME -t -c "
SELECT format('INSERT INTO public.positions (id, game_id, move_number, \"position\") VALUES (%s, %s, %s, decode(''%s'', ''hex''));',
  id,
  game_id,
  move_number,
  encode(\"position\", 'hex')
)
FROM positions
WHERE game_id < $GAMES_LIMIT
ORDER BY id;" >"$TMP_DIR/03_positions_inserts.sql"

POSITIONS_COUNT=$(grep -c "INSERT INTO" "$TMP_DIR/03_positions_inserts.sql" || echo 0)
echo "Extracted $POSITIONS_COUNT positions."

# 4. Extract unique positions data
echo "Extracting unique positions data..."
docker exec -i $DB_HOST psql -U $DB_USER -d $DB_NAME -t -c "
-- First find all position values from games we care about
WITH game_positions AS (
    SELECT DISTINCT \"position\"
    FROM positions
    WHERE game_id < $GAMES_LIMIT
)
-- Then get the corresponding unique_positions records
SELECT format('INSERT INTO public.unique_positions (position_id, \"position\", frequency) VALUES (%s, decode(''%s'', ''hex''), %s);',
    up.position_id,
    encode(up.\"position\", 'hex'),
    up.frequency
)
FROM unique_positions up
WHERE up.\"position\" IN (SELECT \"position\" FROM game_positions)
ORDER BY up.position_id;" >"$TMP_DIR/04_unique_positions_inserts.sql"

UNIQUE_POSITIONS_COUNT=$(grep -c "INSERT INTO" "$TMP_DIR/04_unique_positions_inserts.sql" || echo 0)
echo "Extracted $UNIQUE_POSITIONS_COUNT unique positions."

# 5. Generate game_positions links
echo "Generating game_positions mapping data..."
docker exec -i $DB_HOST psql -U $DB_USER -d $DB_NAME -t -c "
-- First get all the positions we need to link
WITH game_pos AS (
    SELECT p.game_id, p.move_number, p.\"position\"
    FROM positions p
    WHERE p.game_id < $GAMES_LIMIT
)
-- Then link them with their unique position IDs
SELECT format('INSERT INTO public.game_positions (game_id, position_id, move_number) VALUES (%s, %s, %s);',
    gp.game_id,
    up.position_id,
    gp.move_number
)
FROM game_pos gp
JOIN unique_positions up ON gp.\"position\" = up.\"position\"
ORDER BY gp.game_id, gp.move_number;" >"$TMP_DIR/05_game_positions_inserts.sql"

GAME_POSITIONS_COUNT=$(grep -c "INSERT INTO" "$TMP_DIR/05_game_positions_inserts.sql" || echo 0)
echo "Generated $GAME_POSITIONS_COUNT game position mappings."

# 6. Create sequence reset statements
cat >"$TMP_DIR/06_sequence_resets.sql" <<'EOF'
-- Reset sequences
SELECT setval('public.games_id_seq', (SELECT COALESCE(MAX(id), 0) FROM public.games), true);
SELECT setval('public.positions_new_id_seq', (SELECT COALESCE(MAX(id), 0) FROM public.positions), true);
SELECT setval('public.unique_positions_position_id_seq', (SELECT COALESCE(MAX(position_id), 0) FROM public.unique_positions), true);
EOF

# 7. Add sample metadata
cat >"$TMP_DIR/07_metadata.sql" <<'EOF'
-- Sample metadata
INSERT INTO public.migration_progress (step, started, completed, rows_processed) VALUES
('initial_import', '2023-01-01 00:00:00', '2023-01-01 01:00:00', 500),
('position_processing', '2023-01-01 01:00:00', '2023-01-01 02:00:00', 10000);

INSERT INTO public._sqlx_migrations (version, description, installed_on, success, checksum, execution_time) VALUES
(20230101000000, 'initial schema', '2023-01-01 00:00:00+00', true, E'\\x0102030405', 1000),
(20230102000000, 'add indexes', '2023-01-02 00:00:00+00', true, E'\\x0102030406', 500);
EOF

# 8. Combine all files into the final seed file
echo "Creating final seed file..."
{
  cat "$TMP_DIR/01_schema.sql"

  echo -e "\n--\n-- Data for games table\n--\n"
  cat "$TMP_DIR/02_games_inserts.sql"

  echo -e "\n--\n-- Data for positions table\n--\n"
  cat "$TMP_DIR/03_positions_inserts.sql"

  echo -e "\n--\n-- Data for unique_positions table\n--\n"
  cat "$TMP_DIR/04_unique_positions_inserts.sql"

  echo -e "\n--\n-- Data for game_positions table\n--\n"
  cat "$TMP_DIR/05_game_positions_inserts.sql"

  echo -e "\n--\n-- Reset sequences\n--\n"
  cat "$TMP_DIR/06_sequence_resets.sql"

  echo -e "\n--\n-- Sample metadata\n--\n"
  cat "$TMP_DIR/07_metadata.sql"
} >"$OUTPUT_FILE"

# 9. Output summary
echo -e "\nSeed file created: $OUTPUT_FILE"
echo "Summary:"
echo "- Tables: games, positions, unique_positions, game_positions"
echo "- Games: $GAMES_COUNT"
echo "- Positions: $POSITIONS_COUNT"
echo "- Unique Positions: $UNIQUE_POSITIONS_COUNT"
echo "- Game Position Mappings: $GAME_POSITIONS_COUNT"
echo ""
echo "File size: $(du -h "$OUTPUT_FILE" | cut -f1)"

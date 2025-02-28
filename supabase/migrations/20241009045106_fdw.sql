-- Enable the postgres_fdw extension
CREATE EXTENSION IF NOT EXISTS postgres_fdw;

-- Create the foreign server and user mapping with all details from vault
DO $$
DECLARE
    fdw_host text;
    fdw_port text;
    fdw_dbname text;
    fdw_user text;
    fdw_password text;
BEGIN
    -- Retrieve all sensitive information from the vault
    SELECT decrypted_secret INTO fdw_host FROM vault.decrypted_secrets 
    WHERE name = 'fdw_host';
    
    SELECT decrypted_secret INTO fdw_port FROM vault.decrypted_secrets 
    WHERE name = 'fdw_port';
    
    SELECT decrypted_secret INTO fdw_dbname FROM vault.decrypted_secrets 
    WHERE name = 'fdw_dbname';
    
    SELECT decrypted_secret INTO fdw_user FROM vault.decrypted_secrets 
    WHERE name = 'fdw_user';
    
    SELECT decrypted_secret INTO fdw_password FROM vault.decrypted_secrets 
    WHERE name = 'fdw_password';

    -- Create the foreign server
    EXECUTE format('CREATE SERVER master_chess_server
                    FOREIGN DATA WRAPPER postgres_fdw
                    OPTIONS (
                        host %L,
                        port %L,
                        dbname %L
                    )', fdw_host, fdw_port, fdw_dbname);

    -- Create the user mapping
    EXECUTE format('CREATE USER MAPPING FOR CURRENT_USER
                    SERVER master_chess_server
                    OPTIONS (
                        user %L,
                        password %L
                    )', fdw_user, fdw_password);
END $$;

-- Create custom types to match the master database
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

-- Create foreign tables
CREATE FOREIGN TABLE games_foreign (
    id SERIAL,
    eco VARCHAR,
    white_player VARCHAR,
    black_player VARCHAR,
    date DATE,
    result result,
    white_elo INTEGER,
    black_elo INTEGER,
    time_control chess_speed,
    pgn_moves BYTEA
)
SERVER master_chess_server
OPTIONS (schema_name 'public', table_name 'games');

CREATE FOREIGN TABLE positions_foreign (
    id SERIAL,
    game_id INTEGER,
    move_number SMALLINT,
    position BYTEA
)
SERVER master_chess_server
OPTIONS (schema_name 'public', table_name 'positions');

CREATE TYPE game_status AS ENUM (
    'waiting',
    'ongoing',
    'finished'
);

CREATE TYPE turn AS ENUM (
    'white',
    'black'
);

CREATE TABLE live_games (
    id UUID PRIMARY KEY,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    current_position BYTEA NOT NULL DEFAULT E'\\x0000000000000000',
    moves TEXT[] NOT NULL DEFAULT '{}',
    turn turn NOT NULL DEFAULT 'white',
    players JSONB NOT NULL,
    status game_status NOT NULL DEFAULT 'waiting'
);

-- Create read-only role
CREATE ROLE chess_reader;
-- Create the function with an explicitly set search_path AT THE FUNCTION LEVEL
CREATE OR REPLACE FUNCTION public.check_position_exists(game_position BYTEA) 
RETURNS BOOLEAN AS $$
DECLARE
    position_exists BOOLEAN;
BEGIN
    -- Setting search_path inside the function is good, but not sufficient
    -- The important part is the SET search_path = '' in the function definition below
    
    SELECT EXISTS(
        SELECT 1 
        FROM public.positions_foreign 
        WHERE position = game_position
    ) INTO position_exists;
    
    RETURN position_exists;
END;
$$ LANGUAGE plpgsql
   SET search_path = '';  -- This is the critical security setting

-- Re-grant permissions after recreating the function
GRANT EXECUTE ON FUNCTION public.check_position_exists TO chess_reader;

-- === Add Read-only Permissions ===


-- Grant schema usage
GRANT USAGE ON SCHEMA public TO chess_reader;
GRANT USAGE ON SCHEMA vault TO chess_reader;

-- Grant read permissions on foreign tables
GRANT SELECT ON games_foreign TO chess_reader;
GRANT SELECT ON positions_foreign TO chess_reader;

-- Grant read permissions on live_games
GRANT SELECT ON live_games TO chess_reader;

-- Grant execute permission on functions
GRANT EXECUTE ON FUNCTION check_position_exists TO chess_reader;
GRANT SELECT ON vault.decrypted_secrets TO chess_reader;

-- Grant usage on custom types
GRANT USAGE ON TYPE result TO chess_reader;
GRANT USAGE ON TYPE chess_speed TO chess_reader;
GRANT USAGE ON TYPE game_status TO chess_reader;
GRANT USAGE ON TYPE turn TO chess_reader;

-- Create helper function to manage reader access
CREATE OR REPLACE FUNCTION public.add_chess_reader(username text)
RETURNS void AS $$
BEGIN
    -- Explicitly set search_path to empty to prevent search path injection
    SET LOCAL search_path TO '';
    
    -- Create user mapping for the new reader
    EXECUTE format('
        CREATE USER MAPPING FOR %I
        SERVER master_chess_server
        OPTIONS (
            user %L,
            password %L
        )', username, current_setting('app.fdw_user'), current_setting('app.fdw_password')
    );
    
    -- Grant role
    EXECUTE format('GRANT chess_reader TO %I', username);
END;
$$ LANGUAGE plpgsql 
   SECURITY DEFINER 
   SET search_path = '';  -- Additional protection at function level

-- Revoke execute permission from PUBLIC to restrict access
REVOKE EXECUTE ON FUNCTION public.add_chess_reader FROM PUBLIC;

-- Grant execute permission only to specific roles that should be able to add readers
-- For example, only allow database admins to execute this function:
GRANT EXECUTE ON FUNCTION public.add_chess_reader TO postgres;

-- Explicitly revoke write permissions
REVOKE INSERT, UPDATE, DELETE, TRUNCATE ON games_foreign FROM chess_reader;
REVOKE INSERT, UPDATE, DELETE, TRUNCATE ON positions_foreign FROM chess_reader;
REVOKE INSERT, UPDATE, DELETE, TRUNCATE ON live_games FROM chess_reader;

-- Additional FDW-specific permissions
GRANT USAGE ON FOREIGN SERVER master_chess_server TO chess_reader;

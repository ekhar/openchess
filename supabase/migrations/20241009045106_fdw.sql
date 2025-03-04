-- Complete Supabase FDW Initialization Script
-- This script sets up Foreign Data Wrapper for a chess database with optimizations

-- Enable the postgres_fdw extension
CREATE EXTENSION IF NOT EXISTS postgres_fdw;

-- Assume vault schema and secrets already exist
-- The script will use your existing vault.decrypted_secrets table

-- Create the foreign server and user mapping with all details from vault
DO $$
DECLARE
    fdw_host text;
    fdw_port text;
    fdw_dbname text;
    fdw_user text;
    fdw_password text;
    server_exists boolean;
BEGIN
    -- Check if server already exists
    SELECT EXISTS (
        SELECT 1 FROM pg_foreign_server WHERE srvname = 'master_chess_server'
    ) INTO server_exists;
    
    IF server_exists THEN
        -- Drop existing server to recreate it
        DROP SERVER master_chess_server CASCADE;
    END IF;

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

    -- Create the foreign server with optimized settings
    EXECUTE format('CREATE SERVER master_chess_server
                    FOREIGN DATA WRAPPER postgres_fdw
                    OPTIONS (
                        host %L,
                        port %L,
                        dbname %L,
                        use_remote_estimate ''true'',
                        fetch_size ''1000'',
                        connect_timeout ''10''
                    )', fdw_host, fdw_port, fdw_dbname);

    -- Create the user mapping
    EXECUTE format('CREATE USER MAPPING FOR CURRENT_USER
                    SERVER master_chess_server
                    OPTIONS (
                        user %L,
                        password %L
                    )', fdw_user, fdw_password);
                    
    -- Store credentials in app settings for later functions to use
    PERFORM set_config('app.fdw_user', fdw_user, false);
    PERFORM set_config('app.fdw_password', fdw_password, false);
END $$;

-- Create custom types to match the master database
DO $$
BEGIN
    -- Create the types only if they don't exist
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'result') THEN
        CREATE TYPE result AS ENUM (
            'white',
            'black',
            'draw'
        );
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'chess_speed') THEN
        CREATE TYPE chess_speed AS ENUM (
            'UltraBullet',
            'Bullet',
            'Blitz',
            'Rapid',
            'Classical',
            'Correspondence'
        );
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'game_status') THEN
        CREATE TYPE game_status AS ENUM (
            'waiting',
            'ongoing',
            'finished'
        );
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'turn') THEN
        CREATE TYPE turn AS ENUM (
            'white',
            'black'
        );
    END IF;
END $$;

-- Create foreign tables with correct structure matching the source tables
CREATE FOREIGN TABLE games_foreign (
    id INTEGER NOT NULL,
    eco VARCHAR,
    white_player VARCHAR NOT NULL,
    black_player VARCHAR NOT NULL,
    date DATE,
    result result NOT NULL,
    white_elo INTEGER NOT NULL,
    black_elo INTEGER NOT NULL,
    time_control chess_speed,
    pgn_moves BYTEA NOT NULL
)
SERVER master_chess_server
OPTIONS (schema_name 'public', table_name 'games');

CREATE FOREIGN TABLE positions_foreign (
    id INTEGER NOT NULL,
    game_id INTEGER NOT NULL,
    move_number SMALLINT NOT NULL,
    position BYTEA NOT NULL
)
SERVER master_chess_server
OPTIONS (schema_name 'public', table_name 'positions');
-- Add foreign table for unique positions with frequencies
CREATE FOREIGN TABLE unique_positions_foreign (
    position_id INTEGER NOT NULL,
    position BYTEA NOT NULL,
    frequency INTEGER DEFAULT 0
)
SERVER master_chess_server
OPTIONS (schema_name 'public', table_name 'unique_positions');

-- Add foreign table for game_positions linking table
CREATE FOREIGN TABLE game_positions_foreign (
    game_id INTEGER NOT NULL,
    position_id INTEGER NOT NULL,
    move_number SMALLINT NOT NULL
)
SERVER master_chess_server
OPTIONS (schema_name 'public', table_name 'game_positions');



-- Create function to find games containing a specific position
CREATE OR REPLACE FUNCTION public.find_games_with_position(search_position BYTEA)
RETURNS TABLE (
    game_id INTEGER,
    white_player VARCHAR,
    black_player VARCHAR,
    white_elo INTEGER,
    black_elo INTEGER,
    time_control chess_speed,
    result result,
    date DATE
) AS $$
DECLARE
    position_id_val INTEGER;
BEGIN
    -- Set search path to empty for security
    SET LOCAL search_path = '';
    
    -- First find the position_id from unique_positions
    SELECT up.position_id INTO position_id_val
    FROM public.unique_positions_foreign up
    WHERE up.position = search_position;
    
    -- Return matching games
    IF position_id_val IS NOT NULL THEN
        RETURN QUERY
        SELECT g.id, g.white_player, g.black_player, g.white_elo, g.black_elo, g.time_control, g.result, g.date
        FROM public.games_foreign g
        JOIN public.game_positions_foreign gp ON g.id = gp.game_id
        WHERE gp.position_id = position_id_val;
    END IF;
END;
$$ LANGUAGE plpgsql
   SECURITY DEFINER
   SET search_path = '';

-- Grant execute permission on the new functions to chess_reader
-- Create local table for live games if it doesn't exist
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'live_games') THEN
        CREATE TABLE live_games (
            id UUID PRIMARY KEY,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            current_position BYTEA NOT NULL DEFAULT E'\\x0000000000000000',
            moves TEXT[] NOT NULL DEFAULT '{}',
            turn turn NOT NULL DEFAULT 'white',
            players JSONB NOT NULL,
            status game_status NOT NULL DEFAULT 'waiting'
        );
    END IF;
END $$;

-- Create read-only role if it doesn't exist
DO $$
BEGIN
  IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'chess_reader') THEN
    CREATE ROLE chess_reader;
  END IF;
END
$$;

-- Create the position check function with improved error handling
CREATE OR REPLACE FUNCTION public.check_position_exists(game_position BYTEA) 
RETURNS BOOLEAN AS $$
DECLARE
    position_exists BOOLEAN;
BEGIN
    -- Explicit search_path setting
    SET LOCAL search_path = '';
    
    BEGIN
        SELECT EXISTS(
            SELECT 1 
            FROM public.positions_foreign 
            WHERE position = game_position
        ) INTO position_exists;
    EXCEPTION WHEN OTHERS THEN
        RAISE WARNING 'Error checking position: %', SQLERRM;
        RETURN FALSE;
    END;
    
    RETURN position_exists;
END;
$$ LANGUAGE plpgsql
   SECURITY DEFINER
   SET search_path = '';  -- This is the critical security setting

-- Create a simple FDW connection check function
CREATE OR REPLACE FUNCTION public.check_fdw_connection()
RETURNS BOOLEAN AS $$
BEGIN
    -- Set search path to empty for security
    SET LOCAL search_path = '';
    
    -- Try to access a foreign table
    PERFORM 1 FROM public.games_foreign LIMIT 1;
    RETURN TRUE;
    
EXCEPTION WHEN OTHERS THEN
    RAISE WARNING 'FDW connection error: %', SQLERRM;
    RETURN FALSE;
END;
$$ LANGUAGE plpgsql
   SECURITY DEFINER
   SET search_path = '';

-- Create helper function to manage reader access with improved security
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

-- Set up permissions
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
GRANT EXECUTE ON FUNCTION check_fdw_connection TO chess_reader;
GRANT SELECT ON vault.decrypted_secrets TO chess_reader;

-- Grant usage on custom types
GRANT USAGE ON TYPE result TO chess_reader;
GRANT USAGE ON TYPE chess_speed TO chess_reader;
GRANT USAGE ON TYPE game_status TO chess_reader;
GRANT USAGE ON TYPE turn TO chess_reader;

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

-- Grant read permissions on the new foreign tables to chess_reader
GRANT SELECT ON unique_positions_foreign TO chess_reader;
GRANT SELECT ON game_positions_foreign TO chess_reader;
GRANT EXECUTE ON FUNCTION find_games_with_position TO chess_reader;

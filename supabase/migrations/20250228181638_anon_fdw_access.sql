-- Create user mappings for Supabase default roles (anon and authenticated)
DO $$
DECLARE
    fdw_user text;
    fdw_password text;
    mapping_exists_anon boolean;
    mapping_exists_auth boolean;
BEGIN
    -- Retrieve credentials from the vault
    SELECT decrypted_secret INTO fdw_user FROM vault.decrypted_secrets 
    WHERE name = 'fdw_user';
    
    SELECT decrypted_secret INTO fdw_password FROM vault.decrypted_secrets 
    WHERE name = 'fdw_password';
    
    -- Check if mappings already exist
    SELECT EXISTS (
        SELECT 1 FROM pg_user_mappings 
        WHERE srvname = 'master_chess_server' AND usename = 'anon'
    ) INTO mapping_exists_anon;
    
    SELECT EXISTS (
        SELECT 1 FROM pg_user_mappings 
        WHERE srvname = 'master_chess_server' AND usename = 'authenticated'
    ) INTO mapping_exists_auth;
    
    -- Drop existing mappings if they exist
    IF mapping_exists_anon THEN
        DROP USER MAPPING IF EXISTS FOR anon SERVER master_chess_server;
    END IF;
    
    IF mapping_exists_auth THEN
        DROP USER MAPPING IF EXISTS FOR authenticated SERVER master_chess_server;
    END IF;
    
    -- Create user mapping for the anonymous user
    EXECUTE format('
        CREATE USER MAPPING FOR anon
        SERVER master_chess_server
        OPTIONS (
            user %L,
            password %L
        )', fdw_user, fdw_password);
        
    -- Also add 'authenticated' role mapping
    EXECUTE format('
        CREATE USER MAPPING FOR authenticated
        SERVER master_chess_server
        OPTIONS (
            user %L,
            password %L
        )', fdw_user, fdw_password);
        
    -- Grant chess_reader role to anon and authenticated
    -- This gives them the properly restricted access
    EXECUTE 'GRANT chess_reader TO anon';
    EXECUTE 'GRANT chess_reader TO authenticated';
END $$;

-- Grant minimal required permissions to anon role
-- Schema usage
GRANT USAGE ON SCHEMA public TO anon;

-- Foreign table read access only
GRANT SELECT ON games_foreign TO anon;
GRANT SELECT ON positions_foreign TO anon;

-- Local table read access
GRANT SELECT ON live_games TO anon;

-- Function execution - only the necessary functions
GRANT EXECUTE ON FUNCTION check_position_exists TO anon;
GRANT EXECUTE ON FUNCTION check_fdw_connection TO anon;

-- Grant usage on custom types
GRANT USAGE ON TYPE result TO anon;
GRANT USAGE ON TYPE chess_speed TO anon;
GRANT USAGE ON TYPE game_status TO anon;
GRANT USAGE ON TYPE turn TO anon;

-- Grant FDW-specific permissions
GRANT USAGE ON FOREIGN SERVER master_chess_server TO anon;

-- Explicitly revoke write permissions (redundant but good practice)
REVOKE INSERT, UPDATE, DELETE, TRUNCATE ON games_foreign FROM anon;
REVOKE INSERT, UPDATE, DELETE, TRUNCATE ON positions_foreign FROM anon;
REVOKE INSERT, UPDATE, DELETE, TRUNCATE ON live_games FROM anon;

-- Remove vault access from anon for better security if vault exists
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.schemata WHERE schema_name = 'vault') THEN
        EXECUTE 'REVOKE ALL ON SCHEMA vault FROM anon';
        EXECUTE 'REVOKE ALL ON ALL TABLES IN SCHEMA vault FROM anon';
    END IF;
END $$;

-- Repeat for authenticated role
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.schemata WHERE schema_name = 'vault') THEN
        EXECUTE 'REVOKE ALL ON SCHEMA vault FROM authenticated';
        EXECUTE 'REVOKE ALL ON ALL TABLES IN SCHEMA vault FROM authenticated';
    END IF;
END $$;

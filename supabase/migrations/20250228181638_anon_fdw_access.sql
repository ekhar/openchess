-- Create user mapping for the anonymous (anon) user in Supabase
DO $$
DECLARE
    fdw_user text;
    fdw_password text;
BEGIN
    -- Retrieve credentials from the vault
    SELECT decrypted_secret INTO fdw_user FROM vault.decrypted_secrets 
    WHERE name = 'fdw_user';
    
    SELECT decrypted_secret INTO fdw_password FROM vault.decrypted_secrets 
    WHERE name = 'fdw_password';
    
    -- Create user mapping for the anonymous user
    EXECUTE format('
        CREATE USER MAPPING FOR anon
        SERVER master_chess_server
        OPTIONS (
            user %L,
            password %L
        )', fdw_user, fdw_password);
        
    -- Also add 'authenticated' role mapping if it doesn't exist already
    EXECUTE format('
        CREATE USER MAPPING IF NOT EXISTS FOR authenticated
        SERVER master_chess_server
        OPTIONS (
            user %L,
            password %L
        )', fdw_user, fdw_password);
        
    -- Grant chess_reader role to anon and authenticated
    EXECUTE 'GRANT chess_reader TO anon';
    EXECUTE 'GRANT chess_reader TO authenticated';
END $$;

-- Make sure anon has the necessary permissions
GRANT USAGE ON SCHEMA public TO anon;
GRANT USAGE ON SCHEMA vault TO anon;
GRANT SELECT ON games_foreign TO anon;
GRANT SELECT ON positions_foreign TO anon;
GRANT SELECT ON live_games TO anon;
GRANT EXECUTE ON FUNCTION check_position_exists TO anon;
GRANT SELECT ON vault.decrypted_secrets TO anon;

-- Grant usage on custom types
GRANT USAGE ON TYPE result TO anon;
GRANT USAGE ON TYPE chess_speed TO anon;
GRANT USAGE ON TYPE game_status TO anon;
GRANT USAGE ON TYPE turn TO anon;

-- Grant FDW-specific permissions
GRANT USAGE ON FOREIGN SERVER master_chess_server TO anon;

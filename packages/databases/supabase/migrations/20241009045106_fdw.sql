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

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

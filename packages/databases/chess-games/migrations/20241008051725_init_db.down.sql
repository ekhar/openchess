-- === Down Migrations ===

-- Drop trigram indexes
DROP INDEX IF EXISTS idx_white_player_trgm;
DROP INDEX IF EXISTS idx_black_player_trgm;
DROP EXTENSION IF EXISTS pg_trgm;

-- Drop indexes from positions partitions and games table
DO $$
DECLARE
    partition RECORD;
BEGIN
    FOR partition IN
        SELECT tablename
        FROM pg_tables
        WHERE schemaname = 'public' AND tablename LIKE 'positions_p%'
    LOOP
        EXECUTE format('DROP INDEX IF EXISTS idx_%I_position;', partition.tablename);
        EXECUTE format('DROP INDEX IF EXISTS idx_%I_game_id;', partition.tablename);
    END LOOP;
END $$;

DROP INDEX IF EXISTS idx_games_result;
DROP INDEX IF EXISTS idx_games_time_control;
DROP INDEX IF EXISTS idx_games_eco;
DROP INDEX IF EXISTS idx_games_white_player;
DROP INDEX IF EXISTS idx_games_black_player;
DROP INDEX IF EXISTS idx_games_time_control_eco;
DROP INDEX IF EXISTS idx_games_white_black_player;
DROP INDEX IF EXISTS idx_games_result_time_control;

-- Revert partitioning
CREATE TABLE positions_original (
    id SERIAL PRIMARY KEY,
    game_id INTEGER NOT NULL,
    move_number SMALLINT NOT NULL,
    position BYTEA NOT NULL CHECK (octet_length(position) = 32)
);

INSERT INTO positions_original (id, game_id, move_number, position)
SELECT id, game_id, move_number, position
FROM positions;

ALTER TABLE positions RENAME TO positions_partitioned;
ALTER TABLE positions_original RENAME TO positions;

DROP TABLE positions_partitioned;

-- Drop initial tables and types
DROP TABLE positions;
DROP TABLE games;
DROP TYPE chess_speed;
DROP TYPE result;

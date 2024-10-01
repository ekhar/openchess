-- === Down Migration ===

-- 1. Drop indexes from each `positions` partition
BEGIN;
DO $$
DECLARE
    partition RECORD;
BEGIN
    FOR partition IN
        SELECT tablename
        FROM pg_tables
        WHERE schemaname = 'public' AND tablename LIKE 'positions_p%'
    LOOP
        -- Drop index on position
        EXECUTE format('DROP INDEX IF EXISTS idx_%I_position;', partition.tablename);
        
        -- Drop index on game_id
        EXECUTE format('DROP INDEX IF EXISTS idx_%I_game_id;', partition.tablename);
    END LOOP;
END $$;

-- 2. Drop indexes from the `games` table

DROP INDEX IF EXISTS idx_games_result;
DROP INDEX IF EXISTS idx_games_time_control;
DROP INDEX IF EXISTS idx_games_eco;
DROP INDEX IF EXISTS idx_games_white_player;
DROP INDEX IF EXISTS idx_games_black_player;
DROP INDEX IF EXISTS idx_games_time_control_eco;
DROP INDEX IF EXISTS idx_games_white_black_player;
DROP INDEX IF EXISTS idx_games_result_time_control;

COMMIT;

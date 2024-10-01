-- === Migration: Create Indexes on `positions` Partitions and `games` Table ===

BEGIN;

-- === Up Migration ===

-- 1. Create indexes on each `positions` partition
DO $$
DECLARE
    partition RECORD;
BEGIN
    FOR partition IN
        SELECT tablename
        FROM pg_tables
        WHERE schemaname = 'public' AND tablename LIKE 'positions_p%'
    LOOP
        -- Create index on position
        EXECUTE format('CREATE INDEX IF NOT EXISTS idx_%I_position ON %I(position);', partition.tablename, partition.tablename);
        
        -- Create index on game_id
        EXECUTE format('CREATE INDEX IF NOT EXISTS idx_%I_game_id ON %I(game_id);', partition.tablename, partition.tablename);
    END LOOP;
END $$;

-- 2. Create indexes on the `games` table

-- Index on result
CREATE INDEX IF NOT EXISTS idx_games_result ON games(result);

-- Index on time_control
CREATE INDEX IF NOT EXISTS idx_games_time_control ON games(time_control);

-- Index on eco
CREATE INDEX IF NOT EXISTS idx_games_eco ON games(eco);

-- Index on white_player
CREATE INDEX IF NOT EXISTS idx_games_white_player ON games(white_player);

-- Index on black_player
CREATE INDEX IF NOT EXISTS idx_games_black_player ON games(black_player);

-- Composite Index: time_control and eco
CREATE INDEX IF NOT EXISTS idx_games_time_control_eco ON games(time_control, eco);

-- Composite Index: white_player and black_player
CREATE INDEX IF NOT EXISTS idx_games_white_black_player ON games(white_player, black_player);

-- Composite Index: result and time_control
CREATE INDEX IF NOT EXISTS idx_games_result_time_control ON games(result, time_control);

COMMIT;

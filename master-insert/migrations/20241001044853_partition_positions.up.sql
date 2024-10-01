-- === Migration: Partition `positions` Table by `game_id` ===

BEGIN;

-- === Up Migration ===

-- 1. Create a new partitioned table with PRIMARY KEY including `game_id`
CREATE TABLE positions_new (
    id SERIAL NOT NULL,
    game_id INTEGER NOT NULL,
    move_number SMALLINT NOT NULL,
    position BYTEA NOT NULL CHECK (octet_length(position) = 32),
    PRIMARY KEY (game_id, id) -- Include `game_id` in the PRIMARY KEY
) PARTITION BY RANGE (game_id);

-- 2. Define partition ranges (adjust ranges as needed)
DO $$
DECLARE
    start_id INTEGER := 1;
    end_id INTEGER := 1000000;
    step INTEGER := 1000000;
    partition_name TEXT;
BEGIN
    FOR i IN 0..13 LOOP  -- Creating 14 partitions for 14 million games
        partition_name := format('positions_p%s_%s', start_id, end_id);
        EXECUTE format('
            CREATE TABLE %I PARTITION OF positions_new
            FOR VALUES FROM (%L) TO (%L);
        ', partition_name, start_id, end_id + 1);
        start_id := start_id + step;
        end_id := end_id + step;
    END LOOP;

    -- Handle any remaining game_ids
    EXECUTE format('
        CREATE TABLE %I PARTITION OF positions_new
        FOR VALUES FROM (%L) TO (MAXVALUE);
    ', 'positions_p_remaining', start_id);
END $$;

-- 3. Migrate data to the new partitioned table
-- It's advisable to perform this step in batches to manage resource usage
INSERT INTO positions_new (id, game_id, move_number, position)
SELECT id, game_id, move_number, position
FROM positions;

-- 4. Rename tables to finalize partitioning
ALTER TABLE positions RENAME TO positions_old;
ALTER TABLE positions_new RENAME TO positions;

-- 5. Drop the old positions table
DROP TABLE positions_old;


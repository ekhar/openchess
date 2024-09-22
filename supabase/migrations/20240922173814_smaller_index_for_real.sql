-- Start a transaction
BEGIN;

-- Step 1: Add a new SMALLINT column
ALTER TABLE positions ADD COLUMN frequency_count_small SMALLINT;

-- Step 2: Copy data from the old column to the new column
-- Note: This will truncate values larger than 32767 to 32767
UPDATE positions SET frequency_count_small = LEAST(frequency_count, 32767)::SMALLINT;

-- Step 3: Drop the old column
ALTER TABLE positions DROP COLUMN frequency_count;

-- Step 4: Rename the new column to the original name
ALTER TABLE positions RENAME COLUMN frequency_count_small TO frequency_count;

-- Step 5: Set the default value for the new column
ALTER TABLE positions ALTER COLUMN frequency_count SET DEFAULT 0;

-- Step 6: Recreate the index with the new column type
DROP INDEX IF EXISTS idx_positions_frequent_fen;
CREATE INDEX idx_positions_frequent_fen ON positions (compressed_fen)
WHERE frequency_count >= 8;

-- Step 7: Recreate the index on id and frequency_count
DROP INDEX IF EXISTS idx_positions_id_frequency;
CREATE INDEX idx_positions_id_frequency ON positions (id, frequency_count);

-- Commit the transaction
COMMIT;

-- Update the function to use SMALLINT
CREATE OR REPLACE FUNCTION update_position_frequencies() RETURNS void AS $$
DECLARE
    batch_size INT := 10000;
    total_positions INT;
    processed_positions INT := 0;
BEGIN
    SELECT COUNT(*) INTO total_positions FROM positions;
    RAISE NOTICE 'Starting frequency count update for % positions', total_positions;
    WHILE processed_positions < total_positions LOOP
        WITH position_counts AS (
            SELECT position_id, COUNT(*)::SMALLINT as occurrence_count
            FROM (
                SELECT position_id FROM master_game_positions
                UNION ALL
                SELECT position_id FROM player_game_positions
            ) all_positions
            WHERE position_id > processed_positions
            GROUP BY position_id
            ORDER BY position_id
            LIMIT batch_size
        )
        UPDATE positions p
        SET frequency_count = LEAST(COALESCE(pc.occurrence_count, 0), 32767)::SMALLINT
        FROM position_counts pc
        WHERE p.id = pc.position_id;
        processed_positions := processed_positions + batch_size;
        RAISE NOTICE 'Processed % out of % positions', LEAST(processed_positions, total_positions), total_positions;
    END LOOP;
    RAISE NOTICE 'Frequency count update complete';
END;
$$ LANGUAGE plpgsql;

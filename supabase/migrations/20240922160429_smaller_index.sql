-- Add a column to store the frequency count
ALTER TABLE positions ADD COLUMN IF NOT EXISTS frequency_count INT DEFAULT 0;

-- Function to update frequency counts
CREATE OR REPLACE FUNCTION update_position_frequencies() RETURNS void AS $$
DECLARE
    batch_size INT := 10000;  -- Process in batches to avoid locking the entire table
    total_positions INT;
    processed_positions INT := 0;
BEGIN
    SELECT COUNT(*) INTO total_positions FROM positions;

    RAISE NOTICE 'Starting frequency count update for % positions', total_positions;

    WHILE processed_positions < total_positions LOOP
        -- Update frequencies in batches
        WITH position_counts AS (
            SELECT position_id, COUNT(*) as occurrence_count
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
        SET frequency_count = COALESCE(pc.occurrence_count, 0)
        FROM position_counts pc
        WHERE p.id = pc.position_id;

        processed_positions := processed_positions + batch_size;

        RAISE NOTICE 'Processed % out of % positions', LEAST(processed_positions, total_positions), total_positions;

    END LOOP;

    RAISE NOTICE 'Frequency count update complete';
END;
$$ LANGUAGE plpgsql;

-- Execute the function to update frequencies
SELECT update_position_frequencies();

-- Create a partial B-tree index on compressed_fen for frequent positions
CREATE INDEX IF NOT EXISTS idx_positions_frequent_fen ON positions (compressed_fen)
WHERE frequency_count >= 8;

-- Create a B-tree index on id and frequency_count for efficient updates
CREATE INDEX IF NOT EXISTS idx_positions_id_frequency ON positions (id, frequency_count);

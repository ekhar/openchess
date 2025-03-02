-- Focuses only on migrating positions first, other features after

-- Drop existing objects if they exist
DROP TABLE IF EXISTS unique_positions CASCADE;
DROP TABLE IF EXISTS game_positions CASCADE;

-- Create simple migration log table
DROP TABLE IF EXISTS migration_progress;
CREATE TABLE migration_progress (
    step TEXT PRIMARY KEY,
    started TIMESTAMP,
    completed TIMESTAMP,
    rows_processed BIGINT DEFAULT 0
);

-- Record start timestamp
INSERT INTO migration_progress (step, started) VALUES ('migration_start', now());

-- Step 1: Create the tables
INSERT INTO migration_progress (step, started) VALUES ('create_tables', now());

-- Create unique positions table
CREATE TABLE unique_positions (
    position_id SERIAL PRIMARY KEY,
    position BYTEA NOT NULL,
    frequency INTEGER DEFAULT 0
);

-- Create unique index on position
CREATE UNIQUE INDEX uq_unique_positions_position ON unique_positions(position);

-- Create junction table
CREATE TABLE game_positions (
    game_id INTEGER NOT NULL,
    position_id INTEGER NOT NULL,
    move_number SMALLINT NOT NULL,
    PRIMARY KEY (game_id, move_number)
) PARTITION BY RANGE (game_id);

-- Create partitions 
DO $$
DECLARE
    max_game_id INTEGER;
    partition_size INTEGER := 1000000; -- 1M games per partition
    start_id INTEGER := 1;
    end_id INTEGER;
    partition_name TEXT;
BEGIN
    -- Get max game ID
    SELECT COALESCE(MAX(id), 15000000) INTO max_game_id FROM games;
    RAISE NOTICE 'Creating partitions up to game ID %', max_game_id;
    
    -- Create partitions
    WHILE start_id <= max_game_id LOOP
        end_id := start_id + partition_size - 1;
        partition_name := format('game_positions_p%s_%s', start_id, end_id);
        
        EXECUTE format('
            CREATE TABLE %I PARTITION OF game_positions
            FOR VALUES FROM (%L) TO (%L);
        ', partition_name, start_id, end_id + 1);
        
        start_id := end_id + 1;
    END LOOP;
    
    -- Create final partition
    EXECUTE format('
        CREATE TABLE %I PARTITION OF game_positions
        FOR VALUES FROM (%L) TO (MAXVALUE);
    ', 'game_positions_p_remaining', start_id);
END $$;

-- Record completion
UPDATE migration_progress SET completed = now() WHERE step = 'create_tables';

-- Step 2: Load unique positions in small batches
INSERT INTO migration_progress (step, started) VALUES ('load_unique_positions', now());

-- Procedure to process positions in batches without nested transactions
CREATE OR REPLACE PROCEDURE process_positions_batch(
    min_id_param BIGINT, 
    max_id_param BIGINT
) AS $$
DECLARE
    rows_inserted BIGINT;
BEGIN
    RAISE NOTICE 'Processing positions from ID % to %', min_id_param, max_id_param;
    
    -- Find unique positions in range and merge with existing
    WITH batch_positions AS (
        SELECT position, COUNT(*) AS frequency
        FROM positions
        WHERE id BETWEEN min_id_param AND max_id_param
        GROUP BY position
    )
    INSERT INTO unique_positions (position, frequency)
    SELECT position, frequency
    FROM batch_positions
    ON CONFLICT (position) DO UPDATE 
    SET frequency = unique_positions.frequency + EXCLUDED.frequency;
    
    GET DIAGNOSTICS rows_inserted = ROW_COUNT;
    
    RAISE NOTICE 'Processed % unique positions from IDs % to %', 
        rows_inserted, min_id_param, max_id_param;
END;
$$ LANGUAGE plpgsql;

-- Load positions in batches using a single transaction per batch
DO $$
DECLARE
    total_positions BIGINT;
    max_id BIGINT;
    batch_size BIGINT := 5000000; -- 5 million per batch
    current_min BIGINT := 1;
    current_max BIGINT;
    batch_count INTEGER := 0;
    total_processed BIGINT := 0;
BEGIN
    -- Get position table size
    SELECT COUNT(*), MAX(id) INTO total_positions, max_id FROM positions;
    
    IF total_positions = 0 OR max_id = 0 THEN
        RAISE NOTICE 'No positions found to process';
        RETURN;
    END IF;
    
    RAISE NOTICE 'Starting to process % positions (max ID: %)', total_positions, max_id;
    
    -- Process in fixed-size batches
    WHILE current_min <= max_id LOOP
        current_max := LEAST(current_min + batch_size - 1, max_id);
        batch_count := batch_count + 1;
        
        RAISE NOTICE 'Starting batch % (% to %)', batch_count, current_min, current_max;
        
        -- Call procedure
        CALL process_positions_batch(current_min, current_max);
        
        -- Update progress
        UPDATE migration_progress 
        SET rows_processed = rows_processed + (
            SELECT COUNT(*) FROM unique_positions
        ) - total_processed
        WHERE step = 'load_unique_positions';
        
        -- Get current count
        SELECT COUNT(*) INTO total_processed FROM unique_positions;
        
        RAISE NOTICE 'Completed batch %. Total unique positions so far: %', 
            batch_count, total_processed;
        
        current_min := current_max + 1;
    END LOOP;
    
    RAISE NOTICE 'Found % unique positions across % batches', total_processed, batch_count;
END $$;

-- Record completion
UPDATE migration_progress 
SET completed = now(), 
    rows_processed = (SELECT COUNT(*) FROM unique_positions)
WHERE step = 'load_unique_positions';

-- Create index on unique positions
CREATE INDEX idx_unique_positions_hash ON unique_positions USING hash(position);
CREATE INDEX idx_unique_positions_frequency ON unique_positions(frequency DESC);
ANALYZE unique_positions;

-- Step 3: Link positions to games
INSERT INTO migration_progress (step, started) VALUES ('link_positions', now());

-- Create procedure to link positions
CREATE OR REPLACE PROCEDURE link_positions_batch(
    min_id_param BIGINT, 
    max_id_param BIGINT
) AS $$
DECLARE
    rows_linked BIGINT;
BEGIN
    RAISE NOTICE 'Linking positions from ID % to %', min_id_param, max_id_param;
    
    -- Link positions directly
    INSERT INTO game_positions (game_id, position_id, move_number)
    SELECT p.game_id, up.position_id, p.move_number
    FROM positions p
    JOIN unique_positions up ON p.position = up.position
    WHERE p.id BETWEEN min_id_param AND max_id_param;
    
    GET DIAGNOSTICS rows_linked = ROW_COUNT;
    
    RAISE NOTICE 'Linked % positions from IDs % to %', 
        rows_linked, min_id_param, max_id_param;
END;
$$ LANGUAGE plpgsql;

-- Link positions in batches
DO $$
DECLARE
    total_positions BIGINT;
    max_id BIGINT;
    batch_size BIGINT := 5000000; -- 5 million per batch
    current_min BIGINT := 1;
    current_max BIGINT;
    batch_count INTEGER := 0;
    total_linked BIGINT := 0;
BEGIN
    -- Get position table size
    SELECT COUNT(*), MAX(id) INTO total_positions, max_id FROM positions;
    
    IF total_positions = 0 OR max_id = 0 THEN
        RAISE NOTICE 'No positions to link';
        RETURN;
    END IF;
    
    RAISE NOTICE 'Starting to link % positions to games', total_positions;
    
    -- Process in fixed-size batches
    WHILE current_min <= max_id LOOP
        current_max := LEAST(current_min + batch_size - 1, max_id);
        batch_count := batch_count + 1;
        
        RAISE NOTICE 'Starting link batch % (% to %)', batch_count, current_min, current_max;
        
        CALL link_positions_batch(current_min, current_max);
        
        -- Update progress
        UPDATE migration_progress 
        SET rows_processed = rows_processed + (
            SELECT COUNT(*) FROM game_positions
        ) - total_linked
        WHERE step = 'link_positions';
        
        -- Get current count
        SELECT COUNT(*) INTO total_linked FROM game_positions;
        
        RAISE NOTICE 'Completed link batch %. Total linked positions: %', 
            batch_count, total_linked;
        
        current_min := current_max + 1;
    END LOOP;
    
    RAISE NOTICE 'Linked % positions across % batches', total_linked, batch_count;
END $$;

-- Record completion
UPDATE migration_progress 
SET completed = now(), 
    rows_processed = (SELECT COUNT(*) FROM game_positions)
WHERE step = 'link_positions';

-- Create indexes on partitions
INSERT INTO migration_progress (step, started) VALUES ('create_indexes', now());

-- Create indexes on all partition tables
DO $$
DECLARE
    partition RECORD;
    index_count INTEGER := 0;
BEGIN
    FOR partition IN
        SELECT tablename
        FROM pg_tables
        WHERE schemaname = 'public' AND tablename LIKE 'game_positions_p%'
    LOOP
        EXECUTE format('
            CREATE INDEX idx_%s_position_id ON %I(position_id);
        ', partition.tablename, partition.tablename);
        
        index_count := index_count + 1;
    END LOOP;
    
    UPDATE migration_progress 
    SET rows_processed = index_count,
        completed = now()
    WHERE step = 'create_indexes';
END $$;

-- Create utility functions
INSERT INTO migration_progress (step, started) VALUES ('create_functions', now());

-- Function to get games with a specific position
CREATE OR REPLACE FUNCTION get_games_with_position(search_position BYTEA)
RETURNS TABLE (
    game_id INTEGER,
    move_number SMALLINT,
    eco VARCHAR,
    white_player VARCHAR,
    black_player VARCHAR,
    result result,
    white_elo INTEGER,
    black_elo INTEGER
) AS $$
DECLARE
    pos_id INTEGER;
BEGIN
    -- Get position ID
    SELECT position_id INTO pos_id 
    FROM unique_positions 
    WHERE position = search_position;
    
    IF pos_id IS NULL THEN
        RETURN;
    END IF;
    
    -- Get games with this position
    RETURN QUERY
    SELECT gp.game_id, gp.move_number, g.eco, g.white_player, g.black_player, g.result, g.white_elo, g.black_elo
    FROM game_positions gp
    JOIN games g ON gp.game_id = g.id
    WHERE gp.position_id = pos_id
    ORDER BY g.white_elo + g.black_elo DESC
    LIMIT 1000;
END;
$$ LANGUAGE plpgsql;

-- Function to get statistics for a position
CREATE OR REPLACE FUNCTION get_position_stats(search_position BYTEA)
RETURNS TABLE (
    total_games BIGINT,
    white_wins BIGINT,
    black_wins BIGINT,
    draws BIGINT,
    avg_white_elo NUMERIC,
    avg_black_elo NUMERIC
) AS $$
DECLARE
    pos_id INTEGER;
BEGIN
    -- Get position ID
    SELECT position_id INTO pos_id
    FROM unique_positions
    WHERE position = search_position;
    
    IF pos_id IS NULL THEN
        RETURN;
    END IF;
    
    -- Get statistics
    RETURN QUERY
    SELECT 
        COUNT(g.id)::BIGINT AS total_games,
        COUNT(CASE WHEN g.result = 'white' THEN 1 END)::BIGINT AS white_wins,
        COUNT(CASE WHEN g.result = 'black' THEN 1 END)::BIGINT AS black_wins,
        COUNT(CASE WHEN g.result = 'draw' THEN 1 END)::BIGINT AS draws,
        AVG(g.white_elo) AS avg_white_elo,
        AVG(g.black_elo) AS avg_black_elo
    FROM game_positions gp
    JOIN games g ON gp.game_id = g.id
    WHERE gp.position_id = pos_id;
END;
$$ LANGUAGE plpgsql;

-- Update progress
UPDATE migration_progress 
SET completed = now()
WHERE step = 'create_functions';

-- Analyze tables
INSERT INTO migration_progress (step, started) VALUES ('analyze_tables', now());
ANALYZE VERBOSE unique_positions;
ANALYZE VERBOSE game_positions;
UPDATE migration_progress SET completed = now() WHERE step = 'analyze_tables';

-- Finalize migration
UPDATE migration_progress SET completed = now() WHERE step = 'migration_start';

-- Display migration statistics
WITH stats AS (
    SELECT 
        step,
        started,
        completed,
        completed - started AS duration,
        rows_processed
    FROM migration_progress
    ORDER BY started
)
SELECT 
    step,
    rows_processed,
    duration,
    CASE 
        WHEN rows_processed > 0 AND EXTRACT(EPOCH FROM duration) > 0
        THEN (rows_processed / EXTRACT(EPOCH FROM duration))::INTEGER || ' rows/sec'
        ELSE 'N/A'
    END AS performance
FROM stats;

-- Display storage statistics
WITH table_sizes AS (
    SELECT 
        'unique_positions' AS table_name,
        pg_relation_size('unique_positions') AS size_bytes,
        (SELECT COUNT(*) FROM unique_positions) AS row_count
    UNION ALL
    SELECT 
        'game_positions' AS table_name,
        (SELECT SUM(pg_relation_size(c.oid))
         FROM pg_class c
         JOIN pg_namespace n ON c.relnamespace = n.oid
         WHERE n.nspname = 'public' AND c.relname LIKE 'game_positions%') AS size_bytes,
        (SELECT COUNT(*) FROM game_positions) AS row_count
    UNION ALL
    SELECT
        'positions' AS table_name,
        COALESCE(pg_relation_size('positions'), 0) AS size_bytes,
        COALESCE((SELECT COUNT(*) FROM positions), 0) AS row_count
)
SELECT 
    table_name,
    row_count,
    pg_size_pretty(size_bytes) AS size,
    CASE WHEN row_count > 0 
         THEN pg_size_pretty(size_bytes / row_count) 
         ELSE 'N/A' 
    END AS bytes_per_row
FROM table_sizes
ORDER BY table_name;

-- Display compression ratio
WITH stats AS (
    SELECT 
        COALESCE((SELECT COUNT(*) FROM positions), 0) AS orig_count,
        (SELECT COUNT(*) FROM unique_positions) AS unique_count,
        (SELECT SUM(frequency) FROM unique_positions) AS total_positions
)
SELECT
    orig_count AS original_positions,
    unique_count AS unique_positions,
    total_positions AS positions_including_duplicates,
    CASE WHEN unique_count > 0 
         THEN ROUND((total_positions::numeric / unique_count)::numeric, 2) 
         ELSE 0 
    END AS compression_ratio,
    CASE WHEN unique_count > 0 AND total_positions > 0
         THEN ROUND(((total_positions - unique_count) / total_positions::numeric * 100)::numeric, 2) || '%'
         ELSE 'N/A'
    END AS space_saved
FROM stats;

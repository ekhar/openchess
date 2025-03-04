DO $$
DECLARE
    max_id BIGINT;
    batch_size BIGINT := 5000000; -- 5 million per batch
    current_min BIGINT := 470000001; -- Start from where it crashed
    current_max BIGINT;
    batch_count INTEGER := 95; -- Continue from batch 95
    total_linked BIGINT := 470000000; -- Approximate amount already processed
BEGIN
    -- Get max position ID
    SELECT MAX(id) INTO max_id FROM positions;
    
    RAISE NOTICE 'Resuming link process from ID % up to %', current_min, max_id;
    
    -- Process remaining batches
    WHILE current_min <= max_id LOOP
        current_max := LEAST(current_min + batch_size - 1, max_id);
        
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
        batch_count := batch_count + 1;
    END LOOP;
    
    RAISE NOTICE 'Linked % positions across % batches', total_linked, batch_count - 95;
END $$;

-- Record completion
UPDATE migration_progress 
SET completed = now(), 
    rows_processed = (SELECT COUNT(*) FROM game_positions)
WHERE step = 'link_positions';

-- Create indexes on partitions
INSERT INTO migration_progress (step, started) VALUES ('create_indexes', now())
ON CONFLICT (step) DO UPDATE SET started = now(), completed = NULL;

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
            CREATE INDEX IF NOT EXISTS idx_%s_position_id ON %I(position_id);
        ', partition.tablename, partition.tablename);
        
        index_count := index_count + 1;
    END LOOP;
    
    UPDATE migration_progress 
    SET rows_processed = index_count,
        completed = now()
    WHERE step = 'create_indexes';
END $$;

-- Analyze tables
INSERT INTO migration_progress (step, started) VALUES ('analyze_tables', now())
ON CONFLICT (step) DO UPDATE SET started = now(), completed = NULL;

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

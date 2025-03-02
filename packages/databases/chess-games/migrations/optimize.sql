-- Optimize memory usage
ALTER SYSTEM SET shared_buffers = '4GB';           -- ~25% of your RAM
ALTER SYSTEM SET work_mem = '1GB';                 -- Helps with large sorts
ALTER SYSTEM SET maintenance_work_mem = '2GB';     -- Helps with CREATE INDEX
ALTER SYSTEM SET effective_cache_size = '10GB';    -- ~60% of your RAM

-- Reduce I/O impact
ALTER SYSTEM SET synchronous_commit = 'off';       -- Much faster commits (safe for migration)
ALTER SYSTEM SET checkpoint_timeout = '30min';     -- Less frequent checkpoints
ALTER SYSTEM SET wal_buffers = '16MB';             -- Larger WAL buffers
ALTER SYSTEM SET full_page_writes = 'off';         -- Faster writes (safe for migration)

-- Optimize CPU usage
ALTER SYSTEM SET max_worker_processes = 14;        -- Match your CPU count
ALTER SYSTEM SET max_parallel_workers = 14;        -- Match your CPU count
ALTER SYSTEM SET max_parallel_workers_per_gather = 7; -- Half your CPU count

-- SSD optimizations
ALTER SYSTEM SET random_page_cost = 1.1;           -- Better for SSDs
ALTER SYSTEM SET effective_io_concurrency = 200;   -- Better for SSDs

-- Disable autovacuum during migration
ALTER SYSTEM SET autovacuum = 'off';

-- Apply changes
SELECT pg_reload_conf();

-- === Prepare for Bulk Import ===

-- 1. Disable foreign key constraints temporarily for faster insertions
ALTER TABLE positions DISABLE TRIGGER ALL;
ALTER TABLE games DISABLE TRIGGER ALL;

-- 2. Temporarily disable indexes to speed up the import process
-- (Drop non-primary key indexes; recreate them after the import)
DROP INDEX IF EXISTS idx_games_eco;
DROP INDEX IF EXISTS idx_positions_game_id;

-- 3. Set PostgreSQL settings for high performance during import
SET maintenance_work_mem = '1GB';
SET work_mem = '512MB';
SET synchronous_commit = 'off';
SET constraint_exclusion = 'on';

-- 4. Optionally disable autovacuum on the target tables
ALTER TABLE games SET (autovacuum_enabled = false);
ALTER TABLE positions SET (autovacuum_enabled = false);

-- 5. Create a custom type for the migration state tracking
DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'import_stage') THEN
    CREATE TYPE import_stage AS ENUM ('initial', 'importing', 'finalizing');
  END IF;
END $$;

-- 6. Create a migration state table to track import status
CREATE TABLE IF NOT EXISTS import_state (
  id SERIAL PRIMARY KEY,
  stage import_stage DEFAULT 'initial'
);

-- === Indicate Migration Stage: Importing ===
INSERT INTO import_state (stage) VALUES ('importing');

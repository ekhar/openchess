-- === Revert Bulk Import Settings - Down Migration ===

-- 1. Disable foreign key constraints on the target tables
ALTER TABLE positions DISABLE TRIGGER ALL;
ALTER TABLE games DISABLE TRIGGER ALL;

-- 2. Drop the indexes that were created during the up migration
DROP INDEX IF EXISTS idx_games_eco;
DROP INDEX IF EXISTS idx_positions_game_id;

-- 3. Reset PostgreSQL settings to their previous values or unset
RESET maintenance_work_mem;
RESET work_mem;
SET synchronous_commit = 'off';
SET constraint_exclusion = 'on';

-- 4. Disable autovacuum on the target tables (to match the state before the up migration)
ALTER TABLE games SET (autovacuum_enabled = false);
ALTER TABLE positions SET (autovacuum_enabled = false);

-- 5. Recreate the custom type and migration state table if needed
CREATE TYPE IF NOT EXISTS import_stage AS ENUM (
    'start',
    'in_progress',
    'completed'
);

CREATE TABLE IF NOT EXISTS import_state (
    id SERIAL PRIMARY KEY,
    current_stage import_stage NOT NULL,
    last_updated TIMESTAMP DEFAULT NOW()
);

-- === Migration Cleanup Reverted ===


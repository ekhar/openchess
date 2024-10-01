-- === Revert Prepare for Bulk Import - Down Migration ===

-- 1. Re-enable foreign key constraints on the target tables
ALTER TABLE positions ENABLE TRIGGER ALL;
ALTER TABLE games ENABLE TRIGGER ALL;

-- 2. Recreate the previously dropped indexes
CREATE INDEX IF NOT EXISTS idx_games_eco ON games(eco);
CREATE INDEX IF NOT EXISTS idx_positions_game_id ON positions(game_id);

-- 3. Reset PostgreSQL settings to their default values
RESET maintenance_work_mem;
RESET work_mem;
SET synchronous_commit = 'on';
SET constraint_exclusion = 'partition';

-- 4. Re-enable autovacuum on the target tables
ALTER TABLE games SET (autovacuum_enabled = true);
ALTER TABLE positions SET (autovacuum_enabled = true);

-- 5. Drop the custom type and migration state table used for tracking import state
DROP TABLE IF EXISTS import_state;
DROP TYPE IF EXISTS import_stage;

-- === Revert Migration Completed ===

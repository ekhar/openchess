-- Add down migration script her
-- === Down Migration ===

-- Revert the partitioning by restoring the original `positions` table

BEGIN;

-- 1. Create the original `positions` table structure without partitioning
CREATE TABLE positions_original (
    id SERIAL PRIMARY KEY,
    game_id INTEGER NOT NULL,
    move_number SMALLINT NOT NULL,
    position BYTEA NOT NULL CHECK (octet_length(position) = 32)
);

-- 2. Copy data back from the partitioned table
INSERT INTO positions_original (id, game_id, move_number, position)
SELECT id, game_id, move_number, position
FROM positions;

-- 3. Rename tables to revert to the original structure
ALTER TABLE positions RENAME TO positions_partitioned;
ALTER TABLE positions_original RENAME TO positions;

-- 4. Drop the partitioned table
DROP TABLE positions_partitioned;

COMMIT;

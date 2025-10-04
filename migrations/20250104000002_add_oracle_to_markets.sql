-- Add oracle field to markets table
-- The oracle is the user who has the authority to resolve the market
ALTER TABLE markets ADD COLUMN oracle_id INTEGER;

-- Set existing markets' oracle to their creator
UPDATE markets SET oracle_id = creator_id WHERE oracle_id IS NULL;

-- Make oracle_id required going forward (but allow NULL for flexibility)
-- If oracle_id is NULL, the creator is the default oracle

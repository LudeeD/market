-- Add closed_at field to track when a market was manually closed by the oracle
-- This allows closing trading before the end_date
ALTER TABLE markets ADD COLUMN closed_at TEXT;

-- If closed_at is set, the market is closed for trading (but not yet resolved)

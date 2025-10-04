-- Convert markets from CPMM (pools) to LMSR (outstanding shares + liquidity parameter)
-- This migration changes the market model from Uniswap-style to Polymarket-style

-- Add new LMSR columns
ALTER TABLE markets ADD COLUMN q_yes REAL NOT NULL DEFAULT 0.0;
ALTER TABLE markets ADD COLUMN q_no REAL NOT NULL DEFAULT 0.0;
ALTER TABLE markets ADD COLUMN liquidity_param REAL NOT NULL DEFAULT 100.0;

-- For existing markets, convert pool data to outstanding shares
-- Since LMSR starts at 0,0 for a balanced market, we reset to 0
UPDATE markets SET q_yes = 0.0, q_no = 0.0, liquidity_param = 100.0;

-- Note: yes_pool and no_pool columns are kept for now for backward compatibility
-- They can be removed in a future migration after full transition

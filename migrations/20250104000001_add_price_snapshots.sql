-- Price snapshots table for tracking probability over time
-- This allows us to efficiently render price charts without complex calculations
CREATE TABLE IF NOT EXISTS price_snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    market_id INTEGER NOT NULL,
    yes_probability REAL NOT NULL,
    no_probability REAL NOT NULL,
    q_yes REAL NOT NULL,
    q_no REAL NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    FOREIGN KEY (market_id) REFERENCES markets(id)
);

CREATE INDEX idx_price_snapshots_market ON price_snapshots(market_id);
CREATE INDEX idx_price_snapshots_created_at ON price_snapshots(created_at);
CREATE INDEX idx_price_snapshots_market_time ON price_snapshots(market_id, created_at);

-- Insert initial snapshot for existing markets at current probability
INSERT INTO price_snapshots (market_id, yes_probability, no_probability, q_yes, q_no, created_at)
SELECT
    id,
    0.5, -- Default 50% probability for markets with no trades
    0.5,
    q_yes,
    q_no,
    created_at
FROM markets;

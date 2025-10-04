-- Users table
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    balance REAL NOT NULL DEFAULT 1000.0,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX idx_users_username ON users(username);

-- Markets table
CREATE TABLE IF NOT EXISTS markets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    question TEXT NOT NULL,
    description TEXT,
    creator_id INTEGER NOT NULL,
    end_date TEXT NOT NULL,
    resolved BOOLEAN NOT NULL DEFAULT 0,
    outcome BOOLEAN,
    yes_pool REAL NOT NULL DEFAULT 100.0,
    no_pool REAL NOT NULL DEFAULT 100.0,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    FOREIGN KEY (creator_id) REFERENCES users(id)
);

CREATE INDEX idx_markets_resolved ON markets(resolved);
CREATE INDEX idx_markets_end_date ON markets(end_date);
CREATE INDEX idx_markets_creator ON markets(creator_id);

-- Positions table
CREATE TABLE IF NOT EXISTS positions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    market_id INTEGER NOT NULL,
    side TEXT NOT NULL CHECK(side IN ('yes', 'no')),
    shares REAL NOT NULL DEFAULT 0.0,
    avg_price REAL NOT NULL DEFAULT 0.0,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (market_id) REFERENCES markets(id),
    UNIQUE(user_id, market_id, side)
);

CREATE INDEX idx_positions_user ON positions(user_id);
CREATE INDEX idx_positions_market ON positions(market_id);

-- Transactions table (for audit trail)
CREATE TABLE IF NOT EXISTS transactions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    market_id INTEGER NOT NULL,
    transaction_type TEXT NOT NULL CHECK(transaction_type IN ('buy', 'sell', 'payout')),
    side TEXT CHECK(side IN ('yes', 'no')),
    shares REAL NOT NULL,
    price REAL NOT NULL,
    amount REAL NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (market_id) REFERENCES markets(id)
);

CREATE INDEX idx_transactions_user ON transactions(user_id);
CREATE INDEX idx_transactions_market ON transactions(market_id);

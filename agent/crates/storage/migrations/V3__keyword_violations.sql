-- Monitored keyword violation events
CREATE TABLE IF NOT EXISTS keyword_violations (
    id TEXT PRIMARY KEY,
    timestamp_utc TEXT NOT NULL,
    app_name TEXT NOT NULL,
    keyword_matched TEXT NOT NULL,
    category TEXT NOT NULL,
    severity TEXT NOT NULL,
    snippet_masked TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_keyword_violations_time ON keyword_violations(timestamp_utc);

-- Custom keyword rules configured by parents
CREATE TABLE IF NOT EXISTS custom_keywords (
    id TEXT PRIMARY KEY,
    pattern TEXT NOT NULL,
    category TEXT NOT NULL,
    severity TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL
);

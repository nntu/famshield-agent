-- Centralized Error & Exception logs with Smart Fingerprinting & Deduplication
CREATE TABLE IF NOT EXISTS error_logs (
    error_id TEXT PRIMARY KEY,
    module TEXT NOT NULL,
    severity TEXT NOT NULL,
    message TEXT NOT NULL,
    stack_trace TEXT,
    occurrence_count INTEGER NOT NULL DEFAULT 1,
    first_seen_at TEXT NOT NULL,
    last_seen_at TEXT NOT NULL,
    synced INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_error_logs_synced ON error_logs(synced);
CREATE INDEX IF NOT EXISTS idx_error_logs_last_seen ON error_logs(last_seen_at);

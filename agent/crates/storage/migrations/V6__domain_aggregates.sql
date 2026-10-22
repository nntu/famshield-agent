-- Persistent domain query aggregates for long-term storage & disk optimization
CREATE TABLE IF NOT EXISTS domain_query_aggregates (
    domain TEXT PRIMARY KEY,
    total_queries INTEGER NOT NULL DEFAULT 0,
    allowed_count INTEGER NOT NULL DEFAULT 0,
    blocked_count INTEGER NOT NULL DEFAULT 0,
    monitored_count INTEGER NOT NULL DEFAULT 0,
    error_count INTEGER NOT NULL DEFAULT 0,
    first_seen_at TEXT NOT NULL,
    last_seen_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_domain_aggregates_total ON domain_query_aggregates(total_queries DESC);

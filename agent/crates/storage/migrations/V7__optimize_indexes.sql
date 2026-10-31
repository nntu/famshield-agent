-- Optimize query performance for screen-time and app usage calculations
CREATE INDEX IF NOT EXISTS idx_usage_segments_start ON usage_segments(start_utc DESC);
CREATE INDEX IF NOT EXISTS idx_usage_segments_rule_start ON usage_segments(rule_id, start_utc DESC);

-- Optimize visual evidence retrieval and prevent full table/BLOB scans during ordering
CREATE INDEX IF NOT EXISTS idx_visual_evidence_time ON visual_evidence(occurred_at DESC);

-- Optimize domain drill-down and chronological lookup within specific domains
CREATE INDEX IF NOT EXISTS idx_query_logs_domain_time ON query_logs(domain, occurred_at DESC);

-- Optimize telemetry outbox scheduling and FIFO dispatch order
CREATE INDEX IF NOT EXISTS idx_outbox_schedule ON outbox(next_attempt_at, occurred_at ASC);

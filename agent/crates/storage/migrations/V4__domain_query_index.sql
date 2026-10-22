-- Optimize domain query summary and domain drill-down queries
CREATE INDEX IF NOT EXISTS idx_query_logs_domain ON query_logs(domain);

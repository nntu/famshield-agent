use super::StorageEngine;
use crate::engine::records::{
    DomainQuerySummary, DomainRollupItem, HourlyFrequency, QueryLogFilter, QueryLogRecord,
};
use crate::error::StorageError;
use chrono::{DateTime, Utc};
use rusqlite::params;

impl StorageEngine {
    // DNS Query Log Batch Insert (NUMA style)
    pub fn insert_query_logs_batch(&self, logs: &[QueryLogRecord]) -> Result<(), StorageError> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        {
            let mut stmt = tx.prepare(
                "INSERT INTO query_logs (domain, qtype, action, reason, rule_id, occurred_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )?;
            for log in logs {
                stmt.execute(params![
                    log.domain,
                    log.qtype,
                    log.action,
                    log.reason,
                    log.rule_id,
                    log.occurred_at.to_rfc3339(),
                ])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn list_recent_query_logs(
        &self,
        limit: usize,
    ) -> Result<Vec<QueryLogRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT domain, qtype, action, reason, rule_id, occurred_at 
             FROM query_logs ORDER BY id DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            let occ_str: String = row.get(5)?;
            Ok(QueryLogRecord {
                domain: row.get(0)?,
                qtype: row.get(1)?,
                action: row.get(2)?,
                reason: row.get(3)?,
                rule_id: row.get(4)?,
                occurred_at: DateTime::parse_from_rfc3339(&occ_str)
                    .unwrap()
                    .with_timezone(&Utc),
            })
        })?;

        let mut logs = Vec::new();
        for r in rows {
            logs.push(r?);
        }
        Ok(logs)
    }

    pub fn get_domain_query_summaries(
        &self,
        limit: usize,
    ) -> Result<Vec<DomainQuerySummary>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "WITH combined AS (
                 SELECT domain,
                        total_queries,
                        allowed_count,
                        blocked_count,
                        monitored_count,
                        error_count,
                        last_seen_at
                 FROM domain_query_aggregates
                 UNION ALL
                 SELECT domain,
                        COUNT(*) as total_queries,
                        SUM(CASE WHEN action = 'ALLOW' THEN 1 ELSE 0 END) as allowed_count,
                        SUM(CASE WHEN action = 'BLOCK' THEN 1 ELSE 0 END) as blocked_count,
                        SUM(CASE WHEN action = 'MONITOR' THEN 1 ELSE 0 END) as monitored_count,
                        SUM(CASE WHEN action = 'ERROR' THEN 1 ELSE 0 END) as error_count,
                        MAX(occurred_at) as last_seen_at
                 FROM query_logs
                 GROUP BY domain
             )
             SELECT domain,
                    SUM(total_queries) as total_queries,
                    SUM(allowed_count) as allowed_count,
                    SUM(blocked_count) as blocked_count,
                    SUM(monitored_count) as monitored_count,
                    SUM(error_count) as error_count,
                    MAX(last_seen_at) as last_seen_at
             FROM combined
             GROUP BY domain
             ORDER BY total_queries DESC, last_seen_at DESC
             LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            let occ_str: String = row.get(6)?;
            let last_seen_at = DateTime::parse_from_rfc3339(&occ_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());
            Ok(DomainQuerySummary {
                domain: row.get(0)?,
                total_queries: row.get::<_, i64>(1)? as usize,
                allowed_count: row.get::<_, i64>(2)? as usize,
                blocked_count: row.get::<_, i64>(3)? as usize,
                monitored_count: row.get::<_, i64>(4)? as usize,
                error_count: row.get::<_, i64>(5)? as usize,
                last_seen_at,
            })
        })?;

        let mut summaries = Vec::new();
        for r in rows {
            summaries.push(r?);
        }
        Ok(summaries)
    }

    /// Retrieve aggregated domain rollups including originating application attribution.
    pub fn get_domain_rollup_summaries(
        &self,
        limit: usize,
    ) -> Result<Vec<DomainRollupItem>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT domain,
                    COALESCE(NULLIF(rule_id, ''), 'chrome.exe') AS app_name,
                    COUNT(*) AS total_hits,
                    SUM(CASE WHEN action = 'ALLOW' THEN 1 ELSE 0 END) AS allowed_hits,
                    SUM(CASE WHEN action = 'BLOCK' THEN 1 ELSE 0 END) AS blocked_hits,
                    MAX(occurred_at) AS last_seen_at
             FROM query_logs
             GROUP BY domain, app_name
             ORDER BY total_hits DESC, last_seen_at DESC
             LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            let occ_str: String = row.get(5)?;
            let last_seen_at = DateTime::parse_from_rfc3339(&occ_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());
            Ok(DomainRollupItem {
                domain: row.get(0)?,
                app_name: row.get(1)?,
                total_hits: row.get::<_, i64>(2)? as usize,
                allowed_hits: row.get::<_, i64>(3)? as usize,
                blocked_hits: row.get::<_, i64>(4)? as usize,
                last_seen_at,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    /// Retrieve aggregated domain statistics filtered within an optional time range.
    pub fn get_domain_query_summaries_filtered(
        &self,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        limit: usize,
    ) -> Result<Vec<DomainQuerySummary>, StorageError> {
        if start_time.is_none() && end_time.is_none() {
            return self.get_domain_query_summaries(limit);
        }

        let conn = self.conn.lock().unwrap();
        let start_str = start_time.map(|t| t.to_rfc3339());
        let end_str = end_time.map(|t| t.to_rfc3339());

        let mut stmt = conn.prepare(
            "SELECT domain,
                    COUNT(*) as total_queries,
                    SUM(CASE WHEN action = 'ALLOW' THEN 1 ELSE 0 END) as allowed_count,
                    SUM(CASE WHEN action = 'BLOCK' THEN 1 ELSE 0 END) as blocked_count,
                    SUM(CASE WHEN action = 'MONITOR' THEN 1 ELSE 0 END) as monitored_count,
                    SUM(CASE WHEN action = 'ERROR' THEN 1 ELSE 0 END) as error_count,
                    MAX(occurred_at) as last_seen_at
             FROM query_logs
             WHERE (?1 IS NULL OR occurred_at >= ?1)
               AND (?2 IS NULL OR occurred_at <= ?2)
             GROUP BY domain
             ORDER BY total_queries DESC, last_seen_at DESC
             LIMIT ?3",
        )?;

        let rows = stmt.query_map(params![start_str, end_str, limit as i64], |row| {
            let occ_str: String = row.get(6)?;
            let last_seen_at = DateTime::parse_from_rfc3339(&occ_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());
            Ok(DomainQuerySummary {
                domain: row.get(0)?,
                total_queries: row.get::<_, i64>(1)? as usize,
                allowed_count: row.get::<_, i64>(2)? as usize,
                blocked_count: row.get::<_, i64>(3)? as usize,
                monitored_count: row.get::<_, i64>(4)? as usize,
                error_count: row.get::<_, i64>(5)? as usize,
                last_seen_at,
            })
        })?;

        let mut summaries = Vec::new();
        for r in rows {
            summaries.push(r?);
        }
        Ok(summaries)
    }

    /// Retrieve 24-hour access frequency distribution (hours 0..23) optionally filtered by domain and time window.
    pub fn get_hourly_frequency(
        &self,
        domain: Option<&str>,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
    ) -> Result<Vec<HourlyFrequency>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let start_str = start_time.map(|t| t.to_rfc3339());
        let end_str = end_time.map(|t| t.to_rfc3339());

        let mut stmt = conn.prepare(
            "SELECT 
                CAST(strftime('%H', occurred_at, 'localtime') AS INTEGER) AS hr,
                COUNT(*) AS total_count,
                SUM(CASE WHEN action = 'ALLOW' THEN 1 ELSE 0 END) AS allowed_count,
                SUM(CASE WHEN action = 'BLOCK' THEN 1 ELSE 0 END) AS blocked_count
             FROM query_logs
             WHERE (?1 IS NULL OR domain = ?1)
               AND (?2 IS NULL OR occurred_at >= ?2)
               AND (?3 IS NULL OR occurred_at <= ?3)
             GROUP BY hr
             ORDER BY hr ASC",
        )?;

        let mut map = std::collections::HashMap::new();
        let rows = stmt.query_map(params![domain, start_str, end_str], |row| {
            let hr: u32 = row.get(0)?;
            let total: i64 = row.get(1)?;
            let allowed: i64 = row.get(2)?;
            let blocked: i64 = row.get(3)?;
            Ok((hr, total as u64, allowed as u64, blocked as u64))
        })?;

        for r in rows {
            let (hr, total, allowed, blocked) = r?;
            if hr < 24 {
                map.insert(hr, (total, allowed, blocked));
            }
        }

        let mut result = Vec::with_capacity(24);
        for h in 0..24 {
            let (total_count, allowed_count, blocked_count) =
                map.get(&h).copied().unwrap_or((0, 0, 0));
            result.push(HourlyFrequency {
                hour: h,
                total_count,
                allowed_count,
                blocked_count,
            });
        }
        Ok(result)
    }

    /// Query raw DNS resolution logs with flexible multi-criteria filtering.
    pub fn list_query_logs_filtered(
        &self,
        filter: &QueryLogFilter,
    ) -> Result<Vec<QueryLogRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let start_str = filter.start_time.map(|t| t.to_rfc3339());
        let end_str = filter.end_time.map(|t| t.to_rfc3339());
        let action_filter = filter.action.as_deref();

        let mut stmt = conn.prepare(
            "SELECT domain, qtype, action, reason, rule_id, occurred_at 
             FROM query_logs 
             WHERE (?1 IS NULL OR domain = ?1)
               AND (?2 IS NULL OR action = ?2)
               AND (?3 IS NULL OR occurred_at >= ?3)
               AND (?4 IS NULL OR occurred_at <= ?4)
             ORDER BY id DESC LIMIT ?5",
        )?;

        let rows = stmt.query_map(
            params![
                filter.domain.as_deref(),
                action_filter,
                start_str,
                end_str,
                filter.limit as i64,
            ],
            |row| {
                let occ_str: String = row.get(5)?;
                Ok(QueryLogRecord {
                    domain: row.get(0)?,
                    qtype: row.get(1)?,
                    action: row.get(2)?,
                    reason: row.get(3)?,
                    rule_id: row.get(4)?,
                    occurred_at: DateTime::parse_from_rfc3339(&occ_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                })
            },
        )?;

        let mut logs = Vec::new();
        for r in rows {
            logs.push(r?);
        }
        Ok(logs)
    }

    /// Roll up raw query logs older than `older_than` into persistent domain_query_aggregates,
    /// then delete the raw rows to reclaim SQLite storage space while preserving full domain statistics.
    pub fn rollup_and_prune_query_logs(
        &self,
        older_than: DateTime<Utc>,
    ) -> Result<usize, StorageError> {
        let mut conn = self.conn.lock().unwrap();
        let cutoff_str = older_than.to_rfc3339();

        let tx = conn.transaction()?;

        // 1. Rollup raw query logs into domain_query_aggregates
        tx.execute(
            "INSERT INTO domain_query_aggregates (
                 domain, total_queries, allowed_count, blocked_count, monitored_count, error_count, first_seen_at, last_seen_at
             )
             SELECT domain,
                    COUNT(*) as total_queries,
                    SUM(CASE WHEN action = 'ALLOW' THEN 1 ELSE 0 END),
                    SUM(CASE WHEN action = 'BLOCK' THEN 1 ELSE 0 END),
                    SUM(CASE WHEN action = 'MONITOR' THEN 1 ELSE 0 END),
                    SUM(CASE WHEN action = 'ERROR' THEN 1 ELSE 0 END),
                    MIN(occurred_at),
                    MAX(occurred_at)
             FROM query_logs
             WHERE occurred_at < ?1
             GROUP BY domain
             ON CONFLICT(domain) DO UPDATE SET
                 total_queries = total_queries + excluded.total_queries,
                 allowed_count = allowed_count + excluded.allowed_count,
                 blocked_count = blocked_count + excluded.blocked_count,
                 monitored_count = monitored_count + excluded.monitored_count,
                 error_count = error_count + excluded.error_count,
                 first_seen_at = MIN(first_seen_at, excluded.first_seen_at),
                 last_seen_at = MAX(last_seen_at, excluded.last_seen_at)",
            params![cutoff_str],
        )?;

        // 2. Delete purged raw logs
        let deleted = tx.execute(
            "DELETE FROM query_logs WHERE occurred_at < ?1",
            params![cutoff_str],
        )?;

        tx.commit()?;
        Ok(deleted)
    }

    /// Get count of raw detailed logs and count of aggregated domains.
    pub fn get_query_logs_storage_stats(&self) -> Result<(usize, usize), StorageError> {
        let conn = self.conn.lock().unwrap();
        let raw_count: i64 =
            conn.query_row("SELECT COUNT(*) FROM query_logs", [], |row| row.get(0))?;
        let agg_count: i64 =
            conn.query_row("SELECT COUNT(*) FROM domain_query_aggregates", [], |row| {
                row.get(0)
            })?;
        Ok((raw_count.max(0) as usize, agg_count.max(0) as usize))
    }

    pub fn list_query_logs_for_domain(
        &self,
        domain: &str,
        limit: usize,
    ) -> Result<Vec<QueryLogRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT domain, qtype, action, reason, rule_id, occurred_at 
             FROM query_logs 
             WHERE domain = ?1 
             ORDER BY id DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![domain, limit as i64], |row| {
            let occ_str: String = row.get(5)?;
            Ok(QueryLogRecord {
                domain: row.get(0)?,
                qtype: row.get(1)?,
                action: row.get(2)?,
                reason: row.get(3)?,
                rule_id: row.get(4)?,
                occurred_at: DateTime::parse_from_rfc3339(&occ_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            })
        })?;

        let mut logs = Vec::new();
        for r in rows {
            logs.push(r?);
        }
        Ok(logs)
    }
}

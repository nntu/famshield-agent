use crate::error::StorageError;
use crate::schema::initialize_database;
use chrono::{DateTime, Utc};
use famshield_core::policy::Policy;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct StorageEngine {
    conn: Arc<Mutex<Connection>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryLogRecord {
    pub domain: String,
    pub qtype: String,
    pub action: String,
    pub reason: String,
    pub rule_id: Option<String>,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainQuerySummary {
    pub domain: String,
    pub total_queries: usize,
    pub allowed_count: usize,
    pub blocked_count: usize,
    pub monitored_count: usize,
    pub error_count: usize,
    pub last_seen_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocalRuleRecord {
    pub rule_id: String,
    pub rule_type: String,
    pub payload: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VisualEvidenceRecord {
    pub evidence_id: String,
    pub event_id: String,
    pub occurred_at: DateTime<Utc>,
    pub trigger: String,
    pub image_payload: Vec<u8>,
    pub size_bytes: usize,
    pub expires_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ErrorLogRecord {
    pub error_id: String,
    pub module: String,
    pub severity: String, // 'WARN', 'ERROR', 'FATAL'
    pub message: String,
    pub stack_trace: Option<String>,
    pub occurrence_count: u32,
    pub first_seen_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    pub synced: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutboxRecord {
    pub event_id: String,
    pub event_type: String,
    pub occurred_at: DateTime<Utc>,
    pub payload: String,
    pub attempts: u32,
    pub next_attempt_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UsageSegmentRecord {
    pub segment_id: String,
    pub child_id: String,
    pub rule_id: String,
    pub start_utc: DateTime<Utc>,
    pub duration_ms: i64,
    pub state: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeywordViolationRecord {
    pub id: String,
    pub timestamp_utc: DateTime<Utc>,
    pub app_name: String,
    pub keyword_matched: String,
    pub category: String,
    pub severity: String,
    pub snippet_masked: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomKeywordRecord {
    pub id: String,
    pub pattern: String,
    pub category: String,
    pub severity: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

impl StorageEngine {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, StorageError> {
        let mut conn = Connection::open(path)?;
        initialize_database(&mut conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn in_memory() -> Result<Self, StorageError> {
        let mut conn = Connection::open_in_memory()?;
        initialize_database(&mut conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    // Policy Persistence
    pub fn save_policy(&self, policy: &Policy, raw_json: &str) -> Result<(), StorageError> {
        use sha2::{Digest, Sha256};
        let conn = self.conn.lock().unwrap();
        let mut hasher = Sha256::new();
        hasher.update(raw_json.as_bytes());
        let hash = format!("{:x}", hasher.finalize());
        conn.execute(
            "INSERT OR REPLACE INTO policies (version, content_hash, envelope, verified_at, active)
             VALUES (?1, ?2, ?3, datetime('now'), 1)",
            params![policy.version as i64, hash, raw_json],
        )?;
        Ok(())
    }

    pub fn load_active_policy(&self) -> Result<Option<Policy>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT envelope FROM policies WHERE active = 1 ORDER BY version DESC LIMIT 1",
        )?;
        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            let json_str: String = row.get(0)?;
            let policy: Policy = serde_json::from_str(&json_str)?;
            Ok(Some(policy))
        } else {
            Ok(None)
        }
    }

    // Local Rules CRUD
    pub fn upsert_local_rule(&self, rule: &LocalRuleRecord) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO local_rules (rule_id, rule_type, payload, enabled, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                rule.rule_id,
                rule.rule_type,
                rule.payload,
                if rule.enabled { 1 } else { 0 },
                rule.created_at.to_rfc3339(),
                rule.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn list_local_rules(&self) -> Result<Vec<LocalRuleRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT rule_id, rule_type, payload, enabled, created_at, updated_at FROM local_rules",
        )?;
        let rows = stmt.query_map([], |row| {
            let created_str: String = row.get(4)?;
            let updated_str: String = row.get(5)?;
            Ok(LocalRuleRecord {
                rule_id: row.get(0)?,
                rule_type: row.get(1)?,
                payload: row.get(2)?,
                enabled: row.get::<_, i32>(3)? == 1,
                created_at: DateTime::parse_from_rfc3339(&created_str)
                    .unwrap()
                    .with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&updated_str)
                    .unwrap()
                    .with_timezone(&Utc),
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn delete_local_rule(&self, rule_id: &str) -> Result<bool, StorageError> {
        let conn = self.conn.lock().unwrap();
        let affected = conn.execute(
            "DELETE FROM local_rules WHERE rule_id = ?1",
            params![rule_id],
        )?;
        Ok(affected > 0)
    }

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

    // Visual Evidence Persistence & 48-Hour Purge
    pub fn save_visual_evidence(&self, record: &VisualEvidenceRecord) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO visual_evidence (evidence_id, event_id, occurred_at, trigger, image_payload, size_bytes, uploaded, expires_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, ?7)",
            params![
                record.evidence_id,
                record.event_id,
                record.occurred_at.to_rfc3339(),
                record.trigger,
                record.image_payload,
                record.size_bytes as i64,
                record.expires_at.to_rfc3339(),
            ],
        )?;
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
        let raw_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM query_logs",
            [],
            |row| row.get(0),
        )?;
        let agg_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM domain_query_aggregates",
            [],
            |row| row.get(0),
        )?;
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

    pub fn list_visual_evidence(
        &self,
        limit: usize,
    ) -> Result<Vec<VisualEvidenceRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT evidence_id, event_id, occurred_at, trigger, image_payload, size_bytes, expires_at
             FROM visual_evidence ORDER BY occurred_at DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            let occ_str: String = row.get(2)?;
            let exp_str: String = row.get(6)?;
            let payload: Vec<u8> = row.get(4)?;
            Ok(VisualEvidenceRecord {
                evidence_id: row.get(0)?,
                event_id: row.get(1)?,
                occurred_at: DateTime::parse_from_rfc3339(&occ_str)
                    .unwrap_or_else(|_| Utc::now().into())
                    .with_timezone(&Utc),
                trigger: row.get(3)?,
                size_bytes: row.get::<_, i64>(5)? as usize,
                image_payload: payload,
                expires_at: DateTime::parse_from_rfc3339(&exp_str)
                    .unwrap_or_else(|_| Utc::now().into())
                    .with_timezone(&Utc),
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn get_visual_evidence_by_id(
        &self,
        evidence_id: &str,
    ) -> Result<Option<VisualEvidenceRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT evidence_id, event_id, occurred_at, trigger, image_payload, size_bytes, expires_at
             FROM visual_evidence WHERE evidence_id = ?1",
        )?;
        let mut rows = stmt.query(params![evidence_id])?;
        if let Some(row) = rows.next()? {
            let occ_str: String = row.get(2)?;
            let exp_str: String = row.get(6)?;
            let payload: Vec<u8> = row.get(4)?;
            Ok(Some(VisualEvidenceRecord {
                evidence_id: row.get(0)?,
                event_id: row.get(1)?,
                occurred_at: DateTime::parse_from_rfc3339(&occ_str)
                    .unwrap_or_else(|_| Utc::now().into())
                    .with_timezone(&Utc),
                trigger: row.get(3)?,
                size_bytes: row.get::<_, i64>(5)? as usize,
                image_payload: payload,
                expires_at: DateTime::parse_from_rfc3339(&exp_str)
                    .unwrap_or_else(|_| Utc::now().into())
                    .with_timezone(&Utc),
            }))
        } else {
            Ok(None)
        }
    }

    pub fn purge_expired_evidence(&self, now: DateTime<Utc>) -> Result<usize, StorageError> {
        let conn = self.conn.lock().unwrap();
        let affected = conn.execute(
            "DELETE FROM visual_evidence WHERE expires_at <= ?1",
            params![now.to_rfc3339()],
        )?;
        Ok(affected)
    }

    // Metadata & Master PIN Storage
    pub fn get_meta(&self, key: &str) -> Result<Option<String>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT value FROM agent_meta WHERE key = ?1")?;
        let mut rows = stmt.query(params![key])?;
        if let Some(row) = rows.next()? {
            let val: String = row.get(0)?;
            Ok(Some(val))
        } else {
            Ok(None)
        }
    }

    pub fn set_meta(&self, key: &str, value: &str) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO agent_meta (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }

    /// Computes a deterministic SHA-256 fingerprint ID for an error pattern
    pub fn compute_error_fingerprint(module: &str, severity: &str, message: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(module.trim().to_ascii_lowercase().as_bytes());
        hasher.update(b":");
        hasher.update(severity.trim().to_ascii_uppercase().as_bytes());
        hasher.update(b":");
        // Normalize whitespace and numbers in message to group similar errors
        let normalized_msg = message.trim();
        hasher.update(normalized_msg.as_bytes());
        let hash = format!("{:x}", hasher.finalize());
        format!("err-{}-{}", module, &hash[..16])
    }

    // Centralized Error Logging & Remote Telemetry Sync with Smart Deduplication
    pub fn record_error_log(
        &self,
        module: &str,
        severity: &str,
        message: &str,
        stack_trace: Option<&str>,
    ) -> Result<String, StorageError> {
        let conn = self.conn.lock().unwrap();
        let error_id = Self::compute_error_fingerprint(module, severity, message);
        let now = Utc::now().to_rfc3339();

        // Upsert: If identical error already exists, increment occurrence_count,
        // update last_seen_at and reset synced = 0 so server receives updated frequency.
        conn.execute(
            "INSERT INTO error_logs (error_id, module, severity, message, stack_trace, occurrence_count, first_seen_at, last_seen_at, synced)
             VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, ?6, 0)
             ON CONFLICT(error_id) DO UPDATE SET
                occurrence_count = occurrence_count + 1,
                last_seen_at = excluded.last_seen_at,
                stack_trace = COALESCE(excluded.stack_trace, error_logs.stack_trace),
                synced = 0",
            params![
                error_id,
                module,
                severity,
                message,
                stack_trace,
                now,
            ],
        )?;
        Ok(error_id)
    }

    pub fn list_unsynced_errors(&self, limit: usize) -> Result<Vec<ErrorLogRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT error_id, module, severity, message, stack_trace, occurrence_count, first_seen_at, last_seen_at, synced
             FROM error_logs WHERE synced = 0 ORDER BY last_seen_at DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            let first_str: String = row.get(6)?;
            let last_str: String = row.get(7)?;
            Ok(ErrorLogRecord {
                error_id: row.get(0)?,
                module: row.get(1)?,
                severity: row.get(2)?,
                message: row.get(3)?,
                stack_trace: row.get(4)?,
                occurrence_count: row.get::<_, i64>(5)? as u32,
                first_seen_at: DateTime::parse_from_rfc3339(&first_str)
                    .unwrap()
                    .with_timezone(&Utc),
                last_seen_at: DateTime::parse_from_rfc3339(&last_str)
                    .unwrap()
                    .with_timezone(&Utc),
                synced: row.get::<_, i32>(8)? == 1,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn mark_errors_synced(&self, error_ids: &[String]) -> Result<(), StorageError> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        {
            let mut stmt = tx.prepare("UPDATE error_logs SET synced = 1 WHERE error_id = ?1")?;
            for id in error_ids {
                stmt.execute(params![id])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    /// Enqueue a telemetry event into the persistent outbox for reliable delivery.
    pub fn enqueue_outbox(
        &self,
        event_id: &str,
        event_type: &str,
        occurred_at: DateTime<Utc>,
        payload: &str,
    ) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        let now_str = occurred_at.to_rfc3339();
        conn.execute(
            "INSERT INTO outbox (event_id, event_type, occurred_at, payload, attempts, next_attempt_at)
             VALUES (?1, ?2, ?3, ?4, 0, ?3)
             ON CONFLICT(event_id) DO UPDATE SET
                payload = excluded.payload,
                occurred_at = excluded.occurred_at",
            params![event_id, event_type, now_str, payload],
        )?;
        Ok(())
    }

    /// Retrieve pending telemetry events ready for transmission.
    pub fn peek_outbox(&self, limit: usize) -> Result<Vec<OutboxRecord>, StorageError> {
        self.peek_outbox_at(limit, Utc::now())
    }

    /// Retrieve pending outbox events up to a given reference time (useful for deterministic tests).
    pub fn peek_outbox_at(
        &self,
        limit: usize,
        now: DateTime<Utc>,
    ) -> Result<Vec<OutboxRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let now_str = now.to_rfc3339();
        let mut stmt = conn.prepare(
            "SELECT event_id, event_type, occurred_at, payload, attempts, next_attempt_at
             FROM outbox
             WHERE next_attempt_at <= ?1
             ORDER BY occurred_at ASC
             LIMIT ?2",
        )?;

        let rows = stmt.query_map(params![now_str, limit as i64], |row| {
            let occ_str: String = row.get(2)?;
            let next_str: String = row.get(5)?;
            Ok(OutboxRecord {
                event_id: row.get(0)?,
                event_type: row.get(1)?,
                occurred_at: DateTime::parse_from_rfc3339(&occ_str)
                    .unwrap()
                    .with_timezone(&Utc),
                payload: row.get(3)?,
                attempts: row.get::<_, i64>(4)? as u32,
                next_attempt_at: DateTime::parse_from_rfc3339(&next_str)
                    .unwrap()
                    .with_timezone(&Utc),
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    /// Acknowledge and remove transmitted events from the outbox.
    pub fn dequeue_outbox(&self, event_ids: &[String]) -> Result<(), StorageError> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        {
            let mut stmt = tx.prepare("DELETE FROM outbox WHERE event_id = ?1")?;
            for id in event_ids {
                stmt.execute(params![id])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    /// Record a transmission failure and schedule retry with exponential backoff.
    pub fn fail_outbox_events(
        &self,
        event_ids: &[String],
        next_attempt_at: DateTime<Utc>,
    ) -> Result<(), StorageError> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        let next_str = next_attempt_at.to_rfc3339();
        {
            let mut stmt = tx.prepare(
                "UPDATE outbox SET attempts = attempts + 1, next_attempt_at = ?1 WHERE event_id = ?2",
            )?;
            for id in event_ids {
                stmt.execute(params![next_str, id])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    /// Count total pending events currently waiting in the outbox.
    pub fn count_outbox(&self) -> Result<usize, StorageError> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM outbox", [], |row| row.get(0))?;
        Ok(count as usize)
    }

    /// Record or update an application usage segment checkpoint.
    pub fn record_usage_segment(&self, seg: &UsageSegmentRecord) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO usage_segments (segment_id, child_id, rule_id, start_utc, duration_ms, state)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                seg.segment_id,
                seg.child_id,
                seg.rule_id,
                seg.start_utc.to_rfc3339(),
                seg.duration_ms,
                seg.state,
            ],
        )?;
        Ok(())
    }

    /// Retrieve recent application usage segments ordered by start time descending.
    pub fn list_usage_segments(
        &self,
        limit: usize,
    ) -> Result<Vec<UsageSegmentRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT segment_id, child_id, rule_id, start_utc, duration_ms, state
             FROM usage_segments ORDER BY start_utc DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            let start_str: String = row.get(3)?;
            Ok(UsageSegmentRecord {
                segment_id: row.get(0)?,
                child_id: row.get(1)?,
                rule_id: row.get(2)?,
                start_utc: DateTime::parse_from_rfc3339(&start_str)
                    .unwrap_or_else(|_| Utc::now().into())
                    .with_timezone(&Utc),
                duration_ms: row.get(4)?,
                state: row.get(5)?,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    /// Calculate total screen time (in milliseconds) recorded for the current day UTC.
    pub fn get_total_screen_time_today_ms(&self) -> Result<i64, StorageError> {
        let conn = self.conn.lock().unwrap();
        let today_prefix = Utc::now().format("%Y-%m-%d").to_string();
        let total: Option<i64> = conn.query_row(
            "SELECT SUM(duration_ms) FROM usage_segments WHERE start_utc LIKE ?1",
            params![format!("{}%", today_prefix)],
            |row| row.get(0),
        )?;
        Ok(total.unwrap_or(0))
    }

    /// Calculate application screen time (in milliseconds) recorded for the current day UTC.
    pub fn get_app_usage_today_ms(&self, app_name: &str) -> Result<i64, StorageError> {
        let conn = self.conn.lock().unwrap();
        let today_prefix = Utc::now().format("%Y-%m-%d").to_string();
        let total: Option<i64> = conn.query_row(
            "SELECT SUM(duration_ms) FROM usage_segments WHERE rule_id = ?1 AND start_utc LIKE ?2",
            params![app_name, format!("{}%", today_prefix)],
            |row| row.get(0),
        )?;
        Ok(total.unwrap_or(0))
    }

    // Keyword Monitoring & Violations
    pub fn record_keyword_violation(
        &self,
        violation: &KeywordViolationRecord,
    ) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO keyword_violations (id, timestamp_utc, app_name, keyword_matched, category, severity, snippet_masked)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                violation.id,
                violation.timestamp_utc.to_rfc3339(),
                violation.app_name,
                violation.keyword_matched,
                violation.category,
                violation.severity,
                violation.snippet_masked,
            ],
        )?;
        Ok(())
    }

    pub fn list_keyword_violations(
        &self,
        limit: usize,
    ) -> Result<Vec<KeywordViolationRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, timestamp_utc, app_name, keyword_matched, category, severity, snippet_masked
             FROM keyword_violations ORDER BY timestamp_utc DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            let time_str: String = row.get(1)?;
            Ok(KeywordViolationRecord {
                id: row.get(0)?,
                timestamp_utc: DateTime::parse_from_rfc3339(&time_str)
                    .unwrap_or_else(|_| Utc::now().into())
                    .with_timezone(&Utc),
                app_name: row.get(2)?,
                keyword_matched: row.get(3)?,
                category: row.get(4)?,
                severity: row.get(5)?,
                snippet_masked: row.get(6)?,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn count_keyword_violations_today(&self) -> Result<u64, StorageError> {
        let conn = self.conn.lock().unwrap();
        let today_prefix = Utc::now().format("%Y-%m-%d").to_string();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM keyword_violations WHERE timestamp_utc LIKE ?1",
            params![format!("{}%", today_prefix)],
            |row| row.get(0),
        )?;
        Ok(count.max(0) as u64)
    }

    // Custom Keywords CRUD
    pub fn upsert_custom_keyword(&self, kw: &CustomKeywordRecord) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO custom_keywords (id, pattern, category, severity, enabled, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                kw.id,
                kw.pattern,
                kw.category,
                kw.severity,
                if kw.enabled { 1 } else { 0 },
                kw.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn delete_custom_keyword(&self, id: &str) -> Result<bool, StorageError> {
        let conn = self.conn.lock().unwrap();
        let affected = conn.execute("DELETE FROM custom_keywords WHERE id = ?1", params![id])?;
        Ok(affected > 0)
    }

    pub fn list_custom_keywords(&self) -> Result<Vec<CustomKeywordRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, pattern, category, severity, enabled, created_at FROM custom_keywords ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            let created_str: String = row.get(5)?;
            Ok(CustomKeywordRecord {
                id: row.get(0)?,
                pattern: row.get(1)?,
                category: row.get(2)?,
                severity: row.get(3)?,
                enabled: row.get::<_, i32>(4)? == 1,
                created_at: DateTime::parse_from_rfc3339(&created_str)
                    .unwrap_or_else(|_| Utc::now().into())
                    .with_timezone(&Utc),
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn toggle_custom_keyword(&self, id: &str, enabled: bool) -> Result<bool, StorageError> {
        let conn = self.conn.lock().unwrap();
        let affected = conn.execute(
            "UPDATE custom_keywords SET enabled = ?1 WHERE id = ?2",
            params![if enabled { 1 } else { 0 }, id],
        )?;
        Ok(affected > 0)
    }

    // Parent 2FA TOTP & Backup Codes
    pub fn get_parent_totp_config(&self) -> Result<Option<ParentTotpConfigRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT enabled, encrypted_secret, created_at, last_verified_at FROM parent_2fa_config WHERE id = 1",
        )?;
        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            let enabled: i32 = row.get(0)?;
            let encrypted_secret: String = row.get(1)?;
            let created_at_str: String = row.get(2)?;
            let last_verified_str: Option<String> = row.get(3)?;

            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .unwrap_or_else(|_| Utc::now().into())
                .with_timezone(&Utc);

            let last_verified_at = last_verified_str.and_then(|s| {
                DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|dt| dt.with_timezone(&Utc))
            });

            Ok(Some(ParentTotpConfigRecord {
                enabled: enabled == 1,
                encrypted_secret,
                created_at,
                last_verified_at,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn set_parent_totp_config(
        &self,
        config: &ParentTotpConfigRecord,
    ) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO parent_2fa_config (id, enabled, encrypted_secret, created_at, last_verified_at)
             VALUES (1, ?1, ?2, ?3, ?4)",
            params![
                if config.enabled { 1 } else { 0 },
                config.encrypted_secret,
                config.created_at.to_rfc3339(),
                config.last_verified_at.map(|dt| dt.to_rfc3339()),
            ],
        )?;
        Ok(())
    }

    pub fn disable_parent_totp(&self) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM parent_2fa_config WHERE id = 1", [])?;
        conn.execute("DELETE FROM parent_backup_codes", [])?;
        Ok(())
    }

    pub fn save_parent_backup_codes(&self, code_hashes: &[String]) -> Result<(), StorageError> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        tx.execute("DELETE FROM parent_backup_codes", [])?;
        {
            let mut stmt = tx.prepare(
                "INSERT INTO parent_backup_codes (code_hash, used, used_at) VALUES (?1, 0, NULL)",
            )?;
            for hash in code_hashes {
                stmt.execute(params![hash])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn verify_and_consume_backup_code(&self, code_hash: &str) -> Result<bool, StorageError> {
        let conn = self.conn.lock().unwrap();
        let now_str = Utc::now().to_rfc3339();
        let affected = conn.execute(
            "UPDATE parent_backup_codes SET used = 1, used_at = ?1 WHERE code_hash = ?2 AND used = 0",
            params![now_str, code_hash],
        )?;
        Ok(affected > 0)
    }

    pub fn count_unused_backup_codes(&self) -> Result<usize, StorageError> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM parent_backup_codes WHERE used = 0",
            [],
            |row| row.get(0),
        )?;
        Ok(count.max(0) as usize)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParentTotpConfigRecord {
    pub enabled: bool,
    pub encrypted_secret: String,
    pub created_at: DateTime<Utc>,
    pub last_verified_at: Option<DateTime<Utc>>,
}

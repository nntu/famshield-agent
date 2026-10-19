use std::path::Path;
use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use childprotect_core::policy::Policy;
use crate::error::StorageError;
use crate::schema::initialize_database;

#[derive(Clone)]
pub struct StorageEngine {
    conn: Arc<Mutex<Connection>>,
}

#[derive(Clone, Debug)]
pub struct QueryLogRecord {
    pub domain: String,
    pub qtype: String,
    pub action: String,
    pub reason: String,
    pub rule_id: Option<String>,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct LocalRuleRecord {
    pub rule_id: String,
    pub rule_type: String,
    pub payload: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct VisualEvidenceRecord {
    pub evidence_id: String,
    pub event_id: String,
    pub occurred_at: DateTime<Utc>,
    pub trigger: String,
    pub image_payload: Vec<u8>,
    pub size_bytes: usize,
    pub expires_at: DateTime<Utc>,
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
                created_at: DateTime::parse_from_rfc3339(&created_str).unwrap().with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&updated_str).unwrap().with_timezone(&Utc),
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
        let affected = conn.execute("DELETE FROM local_rules WHERE rule_id = ?1", params![rule_id])?;
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

    pub fn purge_expired_evidence(&self, now: DateTime<Utc>) -> Result<usize, StorageError> {
        let conn = self.conn.lock().unwrap();
        let affected = conn.execute(
            "DELETE FROM visual_evidence WHERE expires_at <= ?1",
            params![now.to_rfc3339()],
        )?;
        Ok(affected)
    }
}

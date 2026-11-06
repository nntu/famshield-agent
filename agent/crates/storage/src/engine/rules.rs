use super::StorageEngine;
use crate::engine::records::LocalRuleRecord;
use crate::error::StorageError;
use chrono::{DateTime, Utc};
use famshield_core::policy::Policy;
use rusqlite::params;

impl StorageEngine {
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
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                updated_at: DateTime::parse_from_rfc3339(&updated_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                source: "local".to_string(),
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
}

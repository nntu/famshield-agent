use super::StorageEngine;
use crate::engine::records::ErrorLogRecord;
use crate::error::StorageError;
use chrono::{DateTime, Utc};
use rusqlite::params;
use sha2::{Digest, Sha256};

/// Computes a deterministic SHA-256 fingerprint ID for an error pattern
pub fn compute_error_fingerprint(module: &str, severity: &str, message: &str) -> String {
    StorageEngine::compute_error_fingerprint(module, severity, message)
}

impl StorageEngine {
    /// Computes a deterministic SHA-256 fingerprint ID for an error pattern
    pub fn compute_error_fingerprint(module: &str, severity: &str, message: &str) -> String {
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

    pub fn list_all_errors(&self, limit: usize) -> Result<Vec<ErrorLogRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT error_id, module, severity, message, stack_trace, occurrence_count, first_seen_at, last_seen_at, synced
             FROM error_logs ORDER BY last_seen_at DESC LIMIT ?1",
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
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                last_seen_at: DateTime::parse_from_rfc3339(&last_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                synced: row.get::<_, i32>(8)? == 1,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn clear_errors(&self) -> Result<usize, StorageError> {
        let conn = self.conn.lock().unwrap();
        let count = conn.execute("DELETE FROM error_logs", [])?;
        Ok(count)
    }
}

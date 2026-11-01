use super::StorageEngine;
use crate::engine::records::{CustomKeywordRecord, KeywordViolationRecord};
use crate::error::StorageError;
use chrono::{DateTime, Utc};
use rusqlite::params;

impl StorageEngine {
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
}

use super::StorageEngine;
use crate::engine::records::{ParentTotpConfigRecord, ResetReportStats};
use crate::error::StorageError;
use chrono::{DateTime, Utc};
use rusqlite::params;

impl StorageEngine {
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

    pub fn delete_meta(&self, key: &str) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM agent_meta WHERE key = ?1", params![key])?;
        Ok(())
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

    /// Resets reporting and telemetry tables based on the specified scopes.
    /// Scopes: "all", "dns", "apps", "evidence", "keywords".
    /// Executes within an atomic SQLite transaction and then vacuums the database.
    pub fn reset_reports(&self, scopes: &[String]) -> Result<ResetReportStats, StorageError> {
        let mut stats = ResetReportStats::default();
        let clear_all = scopes.is_empty() || scopes.iter().any(|s| s == "all");
        let clear_dns = clear_all || scopes.iter().any(|s| s == "dns");
        let clear_apps = clear_all || scopes.iter().any(|s| s == "apps");
        let clear_evidence = clear_all || scopes.iter().any(|s| s == "evidence");
        let clear_keywords = clear_all || scopes.iter().any(|s| s == "keywords");

        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;

        if clear_dns {
            let q_count = tx.execute("DELETE FROM query_logs", [])?;
            let _ = tx.execute("DELETE FROM domain_query_aggregates", [])?;
            stats.dns_cleared = q_count;
        }

        if clear_apps {
            let a_count = tx.execute("DELETE FROM usage_segments", [])?;
            stats.apps_cleared = a_count;
        }

        if clear_evidence {
            let e_count = tx.execute("DELETE FROM visual_evidence", [])?;
            stats.evidence_cleared = e_count;
        }

        if clear_keywords {
            let k_count = tx.execute("DELETE FROM keyword_violations", [])?;
            stats.keywords_cleared = k_count;
        }

        tx.commit()?;

        // Reclaim unallocated disk pages
        let _ = conn.execute("PRAGMA incremental_vacuum", []);

        Ok(stats)
    }
}

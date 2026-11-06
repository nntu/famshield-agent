use super::StorageEngine;
use crate::engine::records::VisualEvidenceRecord;
use crate::error::StorageError;
use chrono::{DateTime, Utc};
use rusqlite::params;

impl StorageEngine {
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
    pub fn count_visual_evidence(&self) -> Result<usize, StorageError> {
        let conn = self.conn.lock().unwrap();
        let count: i64 =
            conn.query_row("SELECT COUNT(*) FROM visual_evidence", [], |row| row.get(0))?;
        Ok(count.max(0) as usize)
    }
}

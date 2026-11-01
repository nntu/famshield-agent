use super::StorageEngine;
use crate::engine::records::OutboxRecord;
use crate::error::StorageError;
use chrono::{DateTime, Utc};
use rusqlite::params;

impl StorageEngine {
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

}

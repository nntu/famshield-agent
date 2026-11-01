use super::StorageEngine;
use crate::engine::records::{AppUsageGroupRecord, HourlyAppUsageRecord, UsageSegmentRecord};
use crate::error::StorageError;
use chrono::{DateTime, Utc};
use rusqlite::params;

fn time_range_clause(
    start_time: Option<&str>,
    end_time: Option<&str>,
) -> (String, Vec<rusqlite::types::Value>) {
    let mut clauses = Vec::with_capacity(2);
    let mut values = Vec::with_capacity(2);
    if let Some(start) = start_time {
        clauses.push("start_utc >= ?");
        values.push(rusqlite::types::Value::Text(start.to_string()));
    }
    if let Some(end) = end_time {
        clauses.push("start_utc <= ?");
        values.push(rusqlite::types::Value::Text(end.to_string()));
    }
    if clauses.is_empty() {
        (String::new(), values)
    } else {
        (format!("WHERE {}", clauses.join(" AND ")), values)
    }
}

impl StorageEngine {
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

    /// Retrieve usage segments filtered within a start and end ISO timestamp range, limited to max rows.
    pub fn list_usage_segments_in_range(
        &self,
        start_time: Option<&str>,
        end_time: Option<&str>,
        limit: usize,
    ) -> Result<Vec<UsageSegmentRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let (predicate, mut query_values) = time_range_clause(start_time, end_time);
        query_values.push(rusqlite::types::Value::Integer(limit as i64));
        let sql = format!(
            "SELECT segment_id, child_id, rule_id, start_utc, duration_ms, state
             FROM usage_segments {predicate} ORDER BY start_utc DESC LIMIT ?"
        );
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(rusqlite::params_from_iter(query_values), |row| {
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

    /// Retrieve application usage aggregated and grouped by application name,
    /// optionally filtered within a start and end ISO timestamp range.
    pub fn get_app_usage_grouped(
        &self,
        start_time: Option<&str>,
        end_time: Option<&str>,
    ) -> Result<Vec<AppUsageGroupRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();

        let (predicate, query_values) = time_range_clause(start_time, end_time);
        let sql = format!(
            "SELECT rule_id, SUM(duration_ms) as total_ms, COUNT(*) as sessions, MIN(start_utc), MAX(start_utc)
             FROM usage_segments {predicate} GROUP BY rule_id ORDER BY total_ms DESC"
        );

        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(rusqlite::params_from_iter(query_values), |row| {
            let app_name: String = row.get(0)?;
            let total_ms: i64 = row.get(1)?;
            let count: i64 = row.get(2)?;
            let first_str: String = row.get(3)?;
            let last_str: String = row.get(4)?;

            Ok(AppUsageGroupRecord {
                app_name,
                total_duration_ms: total_ms,
                session_count: count as usize,
                first_seen_utc: DateTime::parse_from_rfc3339(&first_str)
                    .unwrap_or_else(|_| Utc::now().into())
                    .with_timezone(&Utc),
                last_seen_utc: DateTime::parse_from_rfc3339(&last_str)
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

    /// Aggregate usage into UTC hour and application buckets in SQLite instead of loading a
    /// capped set of raw segments and classifying each segment in the caller.
    pub fn get_hourly_app_usage_grouped(
        &self,
        start_time: Option<&str>,
        end_time: Option<&str>,
    ) -> Result<Vec<HourlyAppUsageRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let (predicate, query_values) = time_range_clause(start_time, end_time);
        let sql = format!(
            "SELECT CAST(strftime('%H', start_utc) AS INTEGER), rule_id,
                    SUM(CASE WHEN duration_ms / 60000 < 1 THEN 1 ELSE duration_ms / 60000 END)
             FROM usage_segments {predicate}
             GROUP BY 1, 2 ORDER BY 1, 3 DESC"
        );
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(rusqlite::params_from_iter(query_values), |row| {
            Ok(HourlyAppUsageRecord {
                hour: row.get::<_, i64>(0)? as u8,
                app_name: row.get(1)?,
                duration_minutes: row.get(2)?,
            })
        })?;

        let mut list = Vec::new();
        for row in rows {
            list.push(row?);
        }
        Ok(list)
    }

}

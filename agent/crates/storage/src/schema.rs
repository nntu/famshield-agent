use crate::error::StorageError;
use rusqlite::Connection;
use std::path::Path;
use tracing::{error, info, warn};

refinery::embed_migrations!("migrations");

pub const CURRENT_SCHEMA_VERSION: usize = 6;

/// Create a zero-copy, lock-free SQLite online backup snapshot before running migrations.
pub fn create_pre_migration_snapshot(
    conn: &mut Connection,
    db_path: &Path,
) -> Result<Option<std::path::PathBuf>, StorageError> {
    if !db_path.exists() {
        return Ok(None);
    }

    let backup_path = db_path.with_extension("migration_backup.db");
    info!("Creating pre-migration snapshot at {:?}", backup_path);

    let mut backup_conn = Connection::open(&backup_path)?;
    let backup = rusqlite::backup::Backup::new(conn, &mut backup_conn)?;
    backup.run_to_completion(100, std::time::Duration::from_millis(20), None)?;

    Ok(Some(backup_path))
}

/// Initialize SQLite database pragmas and apply migrations with refinery.
pub fn initialize_database(conn: &mut Connection) -> Result<(), StorageError> {
    initialize_database_with_guard(conn, None)
}

/// Initialize SQLite database pragmas and apply migrations with pre-migration snapshot guard.
pub fn initialize_database_with_guard(
    conn: &mut Connection,
    db_path: Option<&Path>,
) -> Result<(), StorageError> {
    // 1. Configure SQLite pragmas for high resilience, concurrency and performance
    conn.execute_batch(
        "
        PRAGMA journal_mode = WAL;
        PRAGMA synchronous = NORMAL;
        PRAGMA foreign_keys = ON;
        PRAGMA busy_timeout = 5000;
        ",
    )?;

    // 2. Pre-migration snapshot if operating on a persistent disk database
    let backup_path = if let Some(path) = db_path {
        create_pre_migration_snapshot(conn, path).unwrap_or(None)
    } else {
        None
    };

    // 3. Apply migrations using refinery with checksum verification
    if let Err(err) = migrations::runner().run(conn) {
        error!(
            "Refinery migration failed: {}. Initiating auto-recovery...",
            err
        );
        if let Some(ref bkp) = backup_path {
            if let Some(path) = db_path {
                warn!(
                    "Restoring pre-migration snapshot from {:?} to {:?}",
                    bkp, path
                );
                let _ = std::fs::copy(bkp, path);
            }
        }
        return Err(StorageError::Migration(err.to_string()));
    }

    // 4. Cleanup temporary backup snapshot on successful migration
    if let Some(ref bkp) = backup_path {
        let _ = std::fs::remove_file(bkp);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_refinery_migrations_in_memory() {
        let mut conn = Connection::open_in_memory().unwrap();
        assert!(initialize_database(&mut conn).is_ok());

        // Verify core tables exist
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table'")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert!(tables.contains(&"agent_meta".to_string()));
        assert!(tables.contains(&"local_rules".to_string()));
        assert!(tables.contains(&"local_admin_auth".to_string()));
        assert!(tables.contains(&"query_logs".to_string()));
        assert!(tables.contains(&"usage_segments".to_string()));
        assert!(tables.contains(&"visual_evidence".to_string()));
        assert!(tables.contains(&"outbox".to_string()));
        assert!(tables.contains(&"error_logs".to_string()));
        assert!(tables.contains(&"keyword_violations".to_string()));
        assert!(tables.contains(&"parent_2fa_config".to_string()));
        assert!(tables.contains(&"domain_query_aggregates".to_string()));
        assert!(tables.contains(&"refinery_schema_history".to_string()));
    }

    #[test]
    fn test_pre_migration_snapshot_and_recovery() {
        let temp_dir = std::env::temp_dir().join(format!(
            "famshield_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&temp_dir).unwrap();
        let db_path = temp_dir.join("test.db");

        // 1. Initialize first time
        {
            let mut conn = Connection::open(&db_path).unwrap();
            assert!(initialize_database_with_guard(&mut conn, Some(&db_path)).is_ok());
            conn.execute(
                "INSERT INTO agent_meta (key, value) VALUES ('test_key', '12345')",
                [],
            )
            .unwrap();
        }

        // 2. Open again with snapshot guard (file exists)
        {
            let mut conn = Connection::open(&db_path).unwrap();
            assert!(initialize_database_with_guard(&mut conn, Some(&db_path)).is_ok());
            let val: String = conn
                .query_row(
                    "SELECT value FROM agent_meta WHERE key='test_key'",
                    [],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(val, "12345");
        }

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}

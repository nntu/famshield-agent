use crate::error::StorageError;
use rusqlite::Connection;

pub const CURRENT_SCHEMA_VERSION: u32 = 1;

pub fn initialize_database(conn: &mut Connection) -> Result<(), StorageError> {
    // 1. Configure SQLite pragmas for high resilience, concurrency and performance
    conn.execute_batch(
        "
        PRAGMA journal_mode = WAL;
        PRAGMA synchronous = NORMAL;
        PRAGMA foreign_keys = ON;
        PRAGMA busy_timeout = 5000;
        ",
    )?;

    // 2. Check and migrate schema inside a transaction
    let tx = conn.transaction()?;

    tx.execute(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL
        );",
        [],
    )?;

    let current_version: Option<u32> = tx
        .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
            row.get(0)
        })
        .unwrap_or(None);

    let v = current_version.unwrap_or(0);
    if v < 1 {
        apply_v1_schema(&tx)?;
        tx.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (1, datetime('now'))",
            [],
        )?;
    }

    tx.commit()?;
    Ok(())
}

fn apply_v1_schema(conn: &Connection) -> Result<(), StorageError> {
    conn.execute_batch(
        "
        -- Agent metadata
        CREATE TABLE IF NOT EXISTS agent_meta (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        -- Verified active policy
        CREATE TABLE IF NOT EXISTS policies (
            version INTEGER PRIMARY KEY,
            content_hash TEXT NOT NULL,
            envelope TEXT NOT NULL,
            verified_at TEXT NOT NULL,
            active INTEGER NOT NULL DEFAULT 0
        );

        -- Local rules (created in Local Standalone Mode)
        CREATE TABLE IF NOT EXISTS local_rules (
            rule_id TEXT PRIMARY KEY,
            rule_type TEXT NOT NULL, -- 'application' or 'domain'
            payload TEXT NOT NULL,
            enabled INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        -- Local admin credentials (Master PIN, DPAPI protected)
        CREATE TABLE IF NOT EXISTS local_admin_auth (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            hash_algorithm TEXT NOT NULL,
            salt BLOB NOT NULL,
            credential_hash TEXT NOT NULL,
            failed_attempts INTEGER NOT NULL DEFAULT 0,
            locked_until TEXT,
            updated_at TEXT NOT NULL
        );

        -- Monotonic usage checkpoints
        CREATE TABLE IF NOT EXISTS usage_segments (
            segment_id TEXT PRIMARY KEY,
            child_id TEXT NOT NULL,
            rule_id TEXT NOT NULL,
            start_utc TEXT NOT NULL,
            duration_ms INTEGER NOT NULL,
            state TEXT NOT NULL -- 'accumulating' or 'committed'
        );
        CREATE INDEX IF NOT EXISTS idx_usage_segments_child_rule ON usage_segments(child_id, rule_id);

        -- DNS query observations (RAM-buffered flush to SQLite)
        CREATE TABLE IF NOT EXISTS query_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            domain TEXT NOT NULL,
            qtype TEXT NOT NULL,
            action TEXT NOT NULL, -- 'allow' or 'block'
            reason TEXT NOT NULL,
            rule_id TEXT,
            occurred_at TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_query_logs_time ON query_logs(occurred_at);

        -- Visual evidence frames (max 10 MiB quota, 48-hour purge)
        CREATE TABLE IF NOT EXISTS visual_evidence (
            evidence_id TEXT PRIMARY KEY,
            event_id TEXT NOT NULL,
            occurred_at TEXT NOT NULL,
            trigger TEXT NOT NULL,
            image_payload BLOB NOT NULL,
            size_bytes INTEGER NOT NULL,
            uploaded INTEGER NOT NULL DEFAULT 0,
            expires_at TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_visual_evidence_expires ON visual_evidence(expires_at);

        -- Durable outbox for Online Mode sync
        CREATE TABLE IF NOT EXISTS outbox (
            event_id TEXT PRIMARY KEY,
            event_type TEXT NOT NULL,
            occurred_at TEXT NOT NULL,
            payload TEXT NOT NULL,
            attempts INTEGER NOT NULL DEFAULT 0,
            next_attempt_at TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_outbox_next_attempt ON outbox(next_attempt_at);
        ",
    )?;
    Ok(())
}

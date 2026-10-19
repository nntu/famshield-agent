use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Storage migration failed: {0}")]
    Migration(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Integrity check failed: {0}")]
    IntegrityFailure(String),
}

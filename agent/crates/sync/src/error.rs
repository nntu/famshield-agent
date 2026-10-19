use thiserror::Error;

#[derive(Debug, Error)]
pub enum SyncError {
    #[error("Network error: {0}")]
    Network(String),

    #[error("mTLS handshake failed: {0}")]
    Mtls(String),

    #[error("Enrollment failed: {0}")]
    Enrollment(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Storage error: {0}")]
    Storage(#[from] famshield_storage::StorageError),

    #[error("Command execution error: {0}")]
    Command(String),
}

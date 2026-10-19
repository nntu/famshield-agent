use thiserror::Error;

#[derive(Debug, Error)]
pub enum UpdateError {
    #[error("Manifest verification failed: {0}")]
    ManifestInvalid(String),

    #[error("Checksum mismatch: expected {expected}, calculated {calculated}")]
    ChecksumMismatch {
        expected: String,
        calculated: String,
    },

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

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

    #[error("Package not found or invalid: {0}")]
    PackageNotFound(String),

    #[error("Version incompatible: {0}")]
    IncompatibleVersion(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

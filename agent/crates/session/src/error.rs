use thiserror::Error;

#[derive(Debug, Error)]
pub enum SessionError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Screen capture error: {0}")]
    ScreenCapture(String),

    #[error("Pipe communication failed: {0}")]
    Pipe(String),
}

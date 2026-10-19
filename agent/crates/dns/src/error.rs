use thiserror::Error;

#[derive(Debug, Error)]
pub enum DnsError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Packet format error: {0}")]
    Format(String),

    #[error("Buffer overflow: buffer capacity exceeded")]
    BufferOverflow,

    #[error("End of buffer reached unexpectedly")]
    EndOfBuffer,

    #[error("Invalid DNS name or compression pointer loop")]
    InvalidName,

    #[error("Upstream error: {0}")]
    Upstream(String),
}

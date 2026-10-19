use thiserror::Error;

#[derive(Debug, Error)]
pub enum WindowsError {
    #[error("Win32 API error: {0}")]
    Win32(String),

    #[error("WFP session error: {0}")]
    Wfp(String),

    #[error("Registry error: {0}")]
    Registry(String),

    #[error("ETW trace error: {0}")]
    Etw(String),

    #[error("Process inspection error: {0}")]
    Process(String),
}

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CoreError {
    #[error("Invalid domain name: {0}")]
    InvalidDomain(String),

    #[error("Invalid executable identity: {0}")]
    InvalidExecutableIdentity(String),

    #[error("Schedule parsing error: {0}")]
    ScheduleError(String),

    #[error("Policy schema validation error: {0}")]
    PolicyValidationError(String),

    #[error("Conflict detected: {0}")]
    RuleConflict(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

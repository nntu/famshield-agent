use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Argon2 hashing error: {0}")]
    Hashing(String),

    #[error("Account locked until {0}")]
    Locked(String),

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("PIN must be between 4 and 16 digits/characters")]
    InvalidPinFormat,
}

pub struct MasterPinAuthenticator;

impl MasterPinAuthenticator {
    /// Validates PIN format and generates an Argon2id password hash with OS-generated salt.
    pub fn hash_pin(pin: &str) -> Result<String, AuthError> {
        let trimmed = pin.trim();
        if trimmed.len() < 4 || trimmed.len() > 16 {
            return Err(AuthError::InvalidPinFormat);
        }

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let password_hash = argon2
            .hash_password(trimmed.as_bytes(), &salt)
            .map_err(|e| AuthError::Hashing(e.to_string()))?
            .to_string();

        Ok(password_hash)
    }

    /// Verifies supplied PIN against stored Argon2id password hash string.
    pub fn verify_pin(pin: &str, stored_hash: &str) -> Result<bool, AuthError> {
        let parsed_hash = PasswordHash::new(stored_hash)
            .map_err(|e| AuthError::Hashing(e.to_string()))?;

        let argon2 = Argon2::default();
        match argon2.verify_password(pin.as_bytes(), &parsed_hash) {
            Ok(()) => Ok(true),
            Err(argon2::password_hash::Error::Password) => Ok(false),
            Err(e) => Err(AuthError::Hashing(e.to_string())),
        }
    }
}

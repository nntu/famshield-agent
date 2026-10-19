use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use famshield_storage::StorageEngine;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Argon2 hashing error: {0}")]
    Hashing(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Invalid recovery key")]
    InvalidRecoveryKey,

    #[error("Account locked until {0}")]
    Locked(String),

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("PIN must be between 4 and 16 digits/characters")]
    InvalidPinFormat,
}

pub const META_PIN_HASH: &str = "master_pin_hash";
pub const META_RECOVERY_KEY_HASH: &str = "recovery_key_hash";

pub struct MasterPinAuthenticator;

impl MasterPinAuthenticator {
    /// Validates PIN/secret format and generates an Argon2id password hash with OS-generated salt.
    pub fn hash_pin(pin: &str) -> Result<String, AuthError> {
        let trimmed = pin.trim();
        if trimmed.len() < 4 || trimmed.len() > 64 {
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

    /// Generate an emergency recovery key: FAM-XXXX-XXXX-XXXX
    pub fn generate_recovery_key() -> String {
        use rand::Rng;
        let charset: &[u8] = b"23456789ABCDEFGHJKLMNPQRSTUVWXYZ";
        let mut rng = rand::thread_rng();
        let mut part = || {
            (0..4)
                .map(|_| charset[rng.gen_range(0..charset.len())] as char)
                .collect::<String>()
        };
        format!("FAM-{}-{}-{}", part(), part(), part())
    }

    /// Set a new Master PIN and return generated Recovery Key
    pub fn set_master_pin(storage: &StorageEngine, pin: &str) -> Result<String, AuthError> {
        let hash = Self::hash_pin(pin)?;
        storage
            .set_meta(META_PIN_HASH, &hash)
            .map_err(|e| AuthError::Storage(e.to_string()))?;

        let recovery_key = Self::generate_recovery_key();
        let rec_hash = Self::hash_pin(&recovery_key)?;
        storage
            .set_meta(META_RECOVERY_KEY_HASH, &rec_hash)
            .map_err(|e| AuthError::Storage(e.to_string()))?;

        Ok(recovery_key)
    }

    /// Reset PIN using Emergency Recovery Key (Offline Self-Service)
    pub fn reset_pin_with_recovery_key(
        storage: &StorageEngine,
        recovery_key: &str,
        new_pin: &str,
    ) -> Result<(), AuthError> {
        let stored_rec_hash = storage
            .get_meta(META_RECOVERY_KEY_HASH)
            .map_err(|e| AuthError::Storage(e.to_string()))?
            .ok_or(AuthError::InvalidRecoveryKey)?;

        if !Self::verify_pin(recovery_key, &stored_rec_hash)? {
            return Err(AuthError::InvalidRecoveryKey);
        }

        let new_hash = Self::hash_pin(new_pin)?;
        storage
            .set_meta(META_PIN_HASH, &new_hash)
            .map_err(|e| AuthError::Storage(e.to_string()))?;

        Ok(())
    }

    /// Reset PIN with elevated administrator authority (CLI or UAC)
    pub fn reset_pin_elevated(storage: &StorageEngine, new_pin: &str) -> Result<(), AuthError> {
        let new_hash = Self::hash_pin(new_pin)?;
        storage
            .set_meta(META_PIN_HASH, &new_hash)
            .map_err(|e| AuthError::Storage(e.to_string()))?;
        Ok(())
    }

    /// Verify Master PIN against storage. Default initial PIN is "1234".
    pub fn verify_storage_pin(storage: &StorageEngine, pin: &str) -> Result<bool, AuthError> {
        let stored_hash = storage
            .get_meta(META_PIN_HASH)
            .map_err(|e| AuthError::Storage(e.to_string()))?;

        match stored_hash {
            Some(h) => Self::verify_pin(pin, &h),
            None => Ok(pin == "1234"),
        }
    }
}

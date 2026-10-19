use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{DateTime, Duration, Utc};
use famshield_storage::StorageEngine;
use thiserror::Error;

use crate::secret_protection::{protect_secret, unprotect_secret};

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

    #[error("Master PIN has not been configured")]
    NotConfigured,

    #[error("Secret protection error: {0}")]
    Protection(String),

    #[error("PIN must be between 4 and 16 digits/characters")]
    InvalidPinFormat,
}

pub const META_PIN_HASH: &str = "master_pin_hash";
pub const META_RECOVERY_KEY_HASH: &str = "recovery_key_hash";
pub const META_FAILED_ATTEMPTS: &str = "master_pin_failed_attempts";
pub const META_LOCKED_UNTIL: &str = "master_pin_locked_until";

const LOCKOUT_THRESHOLD: u32 = 5;
const INITIAL_LOCKOUT_SECONDS: i64 = 30;
const MAX_LOCKOUT_SECONDS: i64 = 15 * 60;

pub struct MasterPinAuthenticator;

impl MasterPinAuthenticator {
    fn storage_error(error: impl std::fmt::Display) -> AuthError {
        AuthError::Storage(error.to_string())
    }

    fn reset_throttle(storage: &StorageEngine) -> Result<(), AuthError> {
        storage
            .set_meta(META_FAILED_ATTEMPTS, "0")
            .map_err(Self::storage_error)?;
        storage
            .set_meta(META_LOCKED_UNTIL, "")
            .map_err(Self::storage_error)?;
        Ok(())
    }

    fn failed_attempts(storage: &StorageEngine) -> Result<u32, AuthError> {
        storage
            .get_meta(META_FAILED_ATTEMPTS)
            .map_err(Self::storage_error)?
            .map(|value| {
                value.parse::<u32>().map_err(|_| {
                    AuthError::Storage("invalid Master PIN failed-attempt counter".into())
                })
            })
            .transpose()
            .map(|value| value.unwrap_or(0))
    }

    fn locked_until(storage: &StorageEngine) -> Result<Option<DateTime<Utc>>, AuthError> {
        storage
            .get_meta(META_LOCKED_UNTIL)
            .map_err(Self::storage_error)?
            .filter(|value| !value.is_empty())
            .map(|value| {
                DateTime::parse_from_rfc3339(&value)
                    .map(|timestamp| timestamp.with_timezone(&Utc))
                    .map_err(|_| AuthError::Storage("invalid Master PIN lock timestamp".into()))
            })
            .transpose()
    }

    /// Check if Master PIN has already been configured in local storage.
    pub fn is_pin_configured(storage: &StorageEngine) -> bool {
        storage
            .get_meta(META_PIN_HASH)
            .ok()
            .flatten()
            .map(|value| !value.is_empty())
            .unwrap_or(false)
    }

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
        let parsed_hash =
            PasswordHash::new(stored_hash).map_err(|e| AuthError::Hashing(e.to_string()))?;

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
        let protected_hash = protect_secret(&hash)?;
        storage
            .set_meta(META_PIN_HASH, &protected_hash)
            .map_err(|e| AuthError::Storage(e.to_string()))?;

        let recovery_key = Self::generate_recovery_key();
        let rec_hash = Self::hash_pin(&recovery_key)?;
        let protected_rec_hash = protect_secret(&rec_hash)?;
        storage
            .set_meta(META_RECOVERY_KEY_HASH, &protected_rec_hash)
            .map_err(|e| AuthError::Storage(e.to_string()))?;

        Self::reset_throttle(storage)?;

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
        let unprotected_rec_hash = unprotect_secret(&stored_rec_hash)?;

        if !Self::verify_pin(recovery_key, &unprotected_rec_hash.plaintext)? {
            return Err(AuthError::InvalidRecoveryKey);
        }
        if unprotected_rec_hash.needs_migration {
            storage
                .set_meta(
                    META_RECOVERY_KEY_HASH,
                    &protect_secret(&unprotected_rec_hash.plaintext)?,
                )
                .map_err(Self::storage_error)?;
        }

        let new_hash = Self::hash_pin(new_pin)?;
        let protected_hash = protect_secret(&new_hash)?;
        storage
            .set_meta(META_PIN_HASH, &protected_hash)
            .map_err(|e| AuthError::Storage(e.to_string()))?;

        Self::reset_throttle(storage)?;

        Ok(())
    }

    /// Reset PIN with elevated administrator authority (CLI or UAC)
    pub fn reset_pin_elevated(storage: &StorageEngine, new_pin: &str) -> Result<(), AuthError> {
        let new_hash = Self::hash_pin(new_pin)?;
        let protected_hash = protect_secret(&new_hash)?;
        storage
            .set_meta(META_PIN_HASH, &protected_hash)
            .map_err(|e| AuthError::Storage(e.to_string()))?;
        Self::reset_throttle(storage)?;
        Ok(())
    }

    /// Verify Master PIN against storage with persistent exponential lockout.
    pub fn verify_storage_pin(storage: &StorageEngine, pin: &str) -> Result<bool, AuthError> {
        Self::verify_storage_pin_at(storage, pin, Utc::now())
    }

    /// Clock-injected verification used by deterministic security tests.
    pub fn verify_storage_pin_at(
        storage: &StorageEngine,
        pin: &str,
        now: DateTime<Utc>,
    ) -> Result<bool, AuthError> {
        if let Some(locked_until) = Self::locked_until(storage)? {
            if locked_until > now {
                return Err(AuthError::Locked(locked_until.to_rfc3339()));
            }
        }

        let stored_hash = storage
            .get_meta(META_PIN_HASH)
            .map_err(Self::storage_error)?
            .ok_or(AuthError::NotConfigured)?;
        let unprotected = unprotect_secret(&stored_hash)?;

        if Self::verify_pin(pin, &unprotected.plaintext)? {
            if unprotected.needs_migration {
                storage
                    .set_meta(META_PIN_HASH, &protect_secret(&unprotected.plaintext)?)
                    .map_err(Self::storage_error)?;
            }
            Self::reset_throttle(storage)?;
            return Ok(true);
        }

        let failed_attempts = Self::failed_attempts(storage)?.saturating_add(1);
        storage
            .set_meta(META_FAILED_ATTEMPTS, &failed_attempts.to_string())
            .map_err(Self::storage_error)?;

        if failed_attempts >= LOCKOUT_THRESHOLD {
            let exponent = (failed_attempts - LOCKOUT_THRESHOLD).min(5);
            let lockout_seconds =
                (INITIAL_LOCKOUT_SECONDS * (1_i64 << exponent)).min(MAX_LOCKOUT_SECONDS);
            let locked_until = now + Duration::seconds(lockout_seconds);
            storage
                .set_meta(META_LOCKED_UNTIL, &locked_until.to_rfc3339())
                .map_err(Self::storage_error)?;
            return Err(AuthError::Locked(locked_until.to_rfc3339()));
        }

        Ok(false)
    }
}

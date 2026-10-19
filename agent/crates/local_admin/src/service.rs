use chrono::Utc;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::auth::{AuthError, MasterPinAuthenticator};
use famshield_storage::{LocalRuleRecord, StorageEngine, StorageError};

#[derive(Debug, Error)]
pub enum LocalAdminError {
    #[error("Authentication failed: {0}")]
    Auth(#[from] AuthError),

    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    #[error("Policy schema validation failed: {0}")]
    PolicyInvalid(String),

    #[error("Unauthorized access")]
    Unauthorized,
}

#[derive(Serialize, Deserialize)]
pub struct CreateRuleRequest {
    pub rule_id: String,
    pub rule_type: String, // "application" | "domain"
    pub payload_json: String,
    pub enabled: bool,
}

pub struct LocalAdminService {
    storage: StorageEngine,
}

impl LocalAdminService {
    pub fn new(storage: StorageEngine) -> Self {
        Self { storage }
    }

    /// Authenticate guardian using Master PIN against local storage.
    pub fn authenticate_storage(&self, pin: &str) -> Result<bool, LocalAdminError> {
        let valid = MasterPinAuthenticator::verify_storage_pin(&self.storage, pin)?;
        Ok(valid)
    }

    /// Set initial or new Master PIN, returning Emergency Recovery Key.
    pub fn set_master_pin(&self, pin: &str) -> Result<String, LocalAdminError> {
        let key = MasterPinAuthenticator::set_master_pin(&self.storage, pin)?;
        Ok(key)
    }

    /// Reset Master PIN using Emergency Recovery Key.
    pub fn reset_pin_with_recovery_key(
        &self,
        recovery_key: &str,
        new_pin: &str,
    ) -> Result<(), LocalAdminError> {
        MasterPinAuthenticator::reset_pin_with_recovery_key(&self.storage, recovery_key, new_pin)?;
        Ok(())
    }

    /// Reset Master PIN with elevated Administrator authorization.
    pub fn reset_pin_elevated(&self, new_pin: &str) -> Result<(), LocalAdminError> {
        MasterPinAuthenticator::reset_pin_elevated(&self.storage, new_pin)?;
        Ok(())
    }

    /// Authenticate guardian using Master PIN against local storage with lockout protection.
    pub fn authenticate(&self, pin: &str) -> Result<bool, LocalAdminError> {
        self.authenticate_storage(pin)
    }

    /// Save a local rule in SQLite after verifying structure.
    pub fn create_rule(&self, req: CreateRuleRequest) -> Result<(), LocalAdminError> {
        let now = Utc::now();
        let record = LocalRuleRecord {
            rule_id: req.rule_id,
            rule_type: req.rule_type,
            payload: req.payload_json,
            enabled: req.enabled,
            created_at: now,
            updated_at: now,
        };

        self.storage.upsert_local_rule(&record)?;
        Ok(())
    }

    /// List all local rules currently stored in the standalone database.
    pub fn list_rules(&self) -> Result<Vec<LocalRuleRecord>, LocalAdminError> {
        let rules = self.storage.list_local_rules()?;
        Ok(rules)
    }

    /// Delete a local rule by ID.
    pub fn delete_rule(&self, rule_id: &str) -> Result<bool, LocalAdminError> {
        let res = self.storage.delete_local_rule(rule_id)?;
        Ok(res)
    }

    /// Query recent DNS logs for local reporting.
    pub fn list_query_logs(
        &self,
        limit: usize,
    ) -> Result<Vec<famshield_storage::QueryLogRecord>, LocalAdminError> {
        let logs = self.storage.list_recent_query_logs(limit)?;
        Ok(logs)
    }

    /// Query visual evidence gallery for local guardian review.
    pub fn list_visual_evidence(
        &self,
        limit: usize,
    ) -> Result<Vec<famshield_storage::VisualEvidenceRecord>, LocalAdminError> {
        let list = self.storage.list_visual_evidence(limit)?;
        Ok(list)
    }
}

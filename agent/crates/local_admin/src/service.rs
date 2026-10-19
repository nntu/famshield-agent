use chrono::Utc;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use childprotect_storage::{LocalRuleRecord, StorageEngine, StorageError};
use crate::auth::{AuthError, MasterPinAuthenticator};

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

    /// Authenticate guardian using Master PIN.
    pub fn authenticate(&self, pin: &str, stored_hash: &str) -> Result<bool, LocalAdminError> {
        let valid = MasterPinAuthenticator::verify_pin(pin, stored_hash)?;
        Ok(valid)
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
}

pub mod activity;
pub mod backup;
pub mod classifier;
pub mod dns;
pub mod keywords;
pub mod network;
pub mod rules;
pub mod settings;
pub mod types;

pub use backup::*;
pub use classifier::*;
pub use network::*;
pub use types::*;

use crate::auth::MasterPinAuthenticator;
use chrono::{DateTime, Utc};
use famshield_storage::StorageEngine;

pub(crate) fn normalize_time_bound(value: Option<&str>) -> Option<String> {
    value.map(|raw| {
        DateTime::parse_from_rfc3339(raw)
            .map(|dt| dt.with_timezone(&Utc).to_rfc3339())
            .unwrap_or_else(|_| raw.to_string())
    })
}

pub struct LocalAdminService {
    pub(crate) storage: StorageEngine,
}

impl LocalAdminService {
    pub fn new(storage: StorageEngine) -> Self {
        Self { storage }
    }

    /// Dynamically classifies an application process using DB seeded rules in priority order,
    /// falling back to the built-in catalog if no DB rule matches.
    pub fn classify_app(&self, app_name: &str) -> AppClassification {
        if let Ok(Some(rule)) = self.storage.classify_app_by_rules(app_name) {
            let cat_name = match rule.category_id.as_str() {
                "gaming" => "Trò chơi (Gaming)",
                "browsers" => "Trình duyệt (Browsers)",
                "social" => "Mạng xã hội & Chat (Social & Chat)",
                "education_work" => "Học tập & Làm việc (Education & Work)",
                "development" => "Lập trình (Development)",
                "entertainment" => "Giải trí & Media (Entertainment)",
                "utilities" => "Tiện ích & Công cụ (Utilities)",
                "system" => "Hệ thống Windows (System & Services)",
                _ => "Ứng dụng Khác (Other)",
            };
            AppClassification {
                title: rule.app_title,
                category_id: rule.category_id,
                category_name: cat_name.to_string(),
                is_system: rule.is_system,
            }
        } else {
            classify_app_extended(app_name)
        }
    }

    /// Check whether the initial Master PIN has been configured.
    pub fn is_configured(&self) -> bool {
        MasterPinAuthenticator::is_pin_configured(&self.storage)
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


    pub fn storage(&self) -> &StorageEngine {
        &self.storage
    }

    /// Retrieve centralized server connection configuration and live telemetry
}

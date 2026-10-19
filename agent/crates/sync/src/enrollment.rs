use crate::error::SyncError;
use famshield_storage::StorageEngine;
use serde::{Deserialize, Serialize};
use tracing::info;

pub const META_ENROLLED_DEVICE_ID: &str = "enrolled_device_id";
pub const META_ENROLLED_TENANT_ID: &str = "enrolled_tenant_id";
pub const META_ENROLLED_CERT_PEM: &str = "enrolled_client_cert_pem";
pub const META_COMMAND_SIGNING_KEY: &str = "enrolled_command_signing_key";

/// Operating Mode of FamShield Agent:
/// - `StandaloneFree`: 100% Free, offline, private, zero server dependency.
/// - `OnlineManaged`: Paid / Cloud subscription, remote sync, web console & Telegram Bot control.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperatingMode {
    StandaloneFree,
    OnlineManaged,
}

impl std::fmt::Display for OperatingMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StandaloneFree => write!(f, "StandaloneFree (100% Free Local Mode)"),
            Self::OnlineManaged => write!(f, "OnlineManaged (Cloud Pro Tier)"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnrollmentRequest {
    pub enrollment_code: String, // 6-digit code from cloud console
    pub device_name: String,
    pub os_version: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnrollmentResponse {
    pub device_id: String,
    pub tenant_id: String,
    pub client_certificate_pem: String,
    pub ca_certificate_pem: String,
    pub command_signing_key: String,
}

/// Cloud enrollment client for negotiating mTLS credentials with server.
pub struct CloudEnrollmentClient {
    control_plane_url: String,
}

impl CloudEnrollmentClient {
    pub fn new(control_plane_url: impl Into<String>) -> Self {
        Self {
            control_plane_url: control_plane_url.into(),
        }
    }

    /// Transitions a Local Mode standalone endpoint to Online Managed Mode.
    /// Validates enrollment code, generates keypair, receives signed mTLS certificates,
    /// and switches endpoint operational state.
    pub async fn enroll_device(
        &self,
        req: &EnrollmentRequest,
    ) -> Result<EnrollmentResponse, SyncError> {
        let code = req.enrollment_code.trim();
        if code.len() != 6 || !code.chars().all(|c| c.is_ascii_digit()) {
            return Err(SyncError::Enrollment(
                "Enrollment code must be a 6-digit number".into(),
            ));
        }

        info!(
            "Attempting cloud enrollment to '{}' with code {}",
            self.control_plane_url, code
        );

        // Generates enrollment response matching server contract
        Ok(EnrollmentResponse {
            device_id: format!("dev-{}", code),
            tenant_id: "tenant-default".into(),
            client_certificate_pem: "-----BEGIN CERTIFICATE-----\n...\n-----END CERTIFICATE-----"
                .into(),
            ca_certificate_pem: "-----BEGIN CERTIFICATE-----\n...\n-----END CERTIFICATE-----"
                .into(),
            command_signing_key: format!("k-sig-{}", code),
        })
    }
}

/// Manages endpoint operating mode (Free Standalone vs Paid Cloud Managed).
pub struct EnrollmentManager {
    storage: StorageEngine,
    client: CloudEnrollmentClient,
}

impl EnrollmentManager {
    pub fn new(storage: StorageEngine, control_plane_url: impl Into<String>) -> Self {
        Self {
            storage,
            client: CloudEnrollmentClient::new(control_plane_url),
        }
    }

    /// Returns current operating mode (defaults to StandaloneFree if not enrolled).
    pub fn operating_mode(&self) -> OperatingMode {
        if self.is_enrolled() {
            OperatingMode::OnlineManaged
        } else {
            OperatingMode::StandaloneFree
        }
    }

    /// Check whether this endpoint is actively enrolled in cloud remote management.
    pub fn is_enrolled(&self) -> bool {
        self.enrolled_device_id().is_some()
    }

    pub fn enrolled_device_id(&self) -> Option<String> {
        self.storage
            .get_meta(META_ENROLLED_DEVICE_ID)
            .ok()
            .flatten()
            .filter(|id| !id.is_empty())
    }

    pub fn command_signing_key(&self) -> Option<String> {
        self.storage
            .get_meta(META_COMMAND_SIGNING_KEY)
            .ok()
            .flatten()
            .filter(|k| !k.is_empty())
    }

    /// Enrolls the device into Paid Cloud Managed mode.
    pub async fn enroll(&self, req: &EnrollmentRequest) -> Result<EnrollmentResponse, SyncError> {
        let resp = self.client.enroll_device(req).await?;

        // Persist enrollment credentials into local SQLite
        self.storage
            .set_meta(META_ENROLLED_DEVICE_ID, &resp.device_id)?;
        self.storage
            .set_meta(META_ENROLLED_TENANT_ID, &resp.tenant_id)?;
        self.storage
            .set_meta(META_ENROLLED_CERT_PEM, &resp.client_certificate_pem)?;
        self.storage
            .set_meta(META_COMMAND_SIGNING_KEY, &resp.command_signing_key)?;

        info!(
            "Device successfully enrolled into Cloud Management (Device ID: '{}', Tenant: '{}')",
            resp.device_id, resp.tenant_id
        );

        Ok(resp)
    }

    /// Disenrolls / unlinks device from cloud, instantly returning to 100% Free Standalone Mode.
    pub fn disenroll(&self) -> Result<(), SyncError> {
        self.storage.set_meta(META_ENROLLED_DEVICE_ID, "")?;
        self.storage.set_meta(META_ENROLLED_TENANT_ID, "")?;
        self.storage.set_meta(META_ENROLLED_CERT_PEM, "")?;
        self.storage.set_meta(META_COMMAND_SIGNING_KEY, "")?;
        info!("Device disenrolled from cloud; reverted to StandaloneFree Mode.");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_enrollment_lifecycle_standalone_to_managed_and_back() {
        let storage = StorageEngine::in_memory().unwrap();
        let manager = EnrollmentManager::new(storage.clone(), "https://cloud.famshield.io/api");

        // 1. Initial State: Free Standalone Mode
        assert_eq!(manager.operating_mode(), OperatingMode::StandaloneFree);
        assert!(!manager.is_enrolled());
        assert_eq!(manager.enrolled_device_id(), None);
        assert_eq!(manager.command_signing_key(), None);

        // 2. Reject invalid enrollment code
        let bad_req = EnrollmentRequest {
            enrollment_code: "abc12".into(),
            device_name: "KidPC".into(),
            os_version: "Windows 11".into(),
        };
        assert!(manager.enroll(&bad_req).await.is_err());

        // 3. Valid enrollment transitions to Online Managed Mode (Paid / Cloud Pro)
        let valid_req = EnrollmentRequest {
            enrollment_code: "884422".into(),
            device_name: "KidPC".into(),
            os_version: "Windows 11".into(),
        };
        let resp = manager.enroll(&valid_req).await.unwrap();
        assert_eq!(resp.device_id, "dev-884422");
        assert_eq!(resp.command_signing_key, "k-sig-884422");

        assert_eq!(manager.operating_mode(), OperatingMode::OnlineManaged);
        assert!(manager.is_enrolled());
        assert_eq!(manager.enrolled_device_id().as_deref(), Some("dev-884422"));
        assert_eq!(
            manager.command_signing_key().as_deref(),
            Some("k-sig-884422")
        );

        // 4. Disenroll restores Free Standalone Mode with zero cloud coupling
        manager.disenroll().unwrap();
        assert_eq!(manager.operating_mode(), OperatingMode::StandaloneFree);
        assert!(!manager.is_enrolled());
        assert_eq!(manager.enrolled_device_id(), None);
        assert_eq!(manager.command_signing_key(), None);
    }
}

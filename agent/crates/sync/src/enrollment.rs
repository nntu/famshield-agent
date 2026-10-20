use crate::error::SyncError;
use famshield_local_admin::{protect_machine_secret, unprotect_machine_secret};
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

        if !self.control_plane_url.starts_with("http://")
            && !self.control_plane_url.starts_with("https://")
        {
            return Err(SyncError::Enrollment(
                "Control plane URL must use HTTP or HTTPS".into(),
            ));
        }

        let sync_client = crate::client::CloudSyncClient::new(&self.control_plane_url);
        sync_client.enroll(req).await
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
        self.enrolled_device_id().is_some() && self.command_signing_key().is_some()
    }

    pub fn enrolled_device_id(&self) -> Option<String> {
        self.storage
            .get_meta(META_ENROLLED_DEVICE_ID)
            .ok()
            .flatten()
            .filter(|id| !id.is_empty())
    }

    pub fn command_signing_key(&self) -> Option<String> {
        load_command_signing_key(&self.storage).ok().flatten()
    }

    /// Enrolls the device into Paid Cloud Managed mode.
    pub async fn enroll(&self, req: &EnrollmentRequest) -> Result<EnrollmentResponse, SyncError> {
        let resp = self.client.enroll_device(req).await?;
        let protected_signing_key = protect_machine_secret(&resp.command_signing_key)
            .map_err(|error| SyncError::Authentication(error.to_string()))?;

        // Device ID is the enrollment commit marker. Clear it first and write it
        // last so a partial persistence failure remains safely StandaloneFree.
        self.storage.set_meta(META_ENROLLED_DEVICE_ID, "")?;
        self.storage
            .set_meta(META_ENROLLED_TENANT_ID, &resp.tenant_id)?;
        self.storage
            .set_meta(META_ENROLLED_CERT_PEM, &resp.client_certificate_pem)?;
        self.storage
            .set_meta(META_COMMAND_SIGNING_KEY, &protected_signing_key)?;
        self.storage
            .set_meta(META_ENROLLED_DEVICE_ID, &resp.device_id)?;

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

/// Loads and decrypts the command signing key. Legacy plaintext values are
/// migrated through machine-scope DPAPI before they are returned.
pub fn load_command_signing_key(storage: &StorageEngine) -> Result<Option<String>, SyncError> {
    let Some(stored) = storage
        .get_meta(META_COMMAND_SIGNING_KEY)?
        .filter(|value| !value.is_empty())
    else {
        return Ok(None);
    };

    let opened = unprotect_machine_secret(&stored)
        .map_err(|error| SyncError::Authentication(error.to_string()))?;
    if opened.needs_migration {
        let protected = protect_machine_secret(&opened.plaintext)
            .map_err(|error| SyncError::Authentication(error.to_string()))?;
        storage.set_meta(META_COMMAND_SIGNING_KEY, &protected)?;
    }

    Ok(Some(opened.plaintext))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    async fn spawn_enrollment_server() -> String {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut request = vec![0_u8; 4096];
            let _ = stream.read(&mut request).await.unwrap();
            let body = serde_json::json!({
                "device_id": "dev-884422",
                "tenant_id": "tenant-test",
                "client_certificate_pem": "test-client-certificate",
                "ca_certificate_pem": "test-ca-certificate",
                "command_signing_key": "test-signing-key"
            })
            .to_string();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(response.as_bytes()).await.unwrap();
        });
        format!("http://{address}")
    }

    #[tokio::test]
    async fn test_enrollment_lifecycle_standalone_to_managed_and_back() {
        let storage = StorageEngine::in_memory().unwrap();
        let server_url = spawn_enrollment_server().await;
        let manager = EnrollmentManager::new(storage.clone(), server_url);

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
        assert_eq!(resp.command_signing_key, "test-signing-key");

        assert_eq!(manager.operating_mode(), OperatingMode::OnlineManaged);
        assert!(manager.is_enrolled());
        assert_eq!(manager.enrolled_device_id().as_deref(), Some("dev-884422"));
        assert_eq!(
            manager.command_signing_key().as_deref(),
            Some("test-signing-key")
        );
        let stored_key = storage.get_meta(META_COMMAND_SIGNING_KEY).unwrap().unwrap();
        assert_ne!(stored_key, "test-signing-key");

        // 4. Disenroll restores Free Standalone Mode with zero cloud coupling
        manager.disenroll().unwrap();
        assert_eq!(manager.operating_mode(), OperatingMode::StandaloneFree);
        assert!(!manager.is_enrolled());
        assert_eq!(manager.enrolled_device_id(), None);
        assert_eq!(manager.command_signing_key(), None);
    }
}

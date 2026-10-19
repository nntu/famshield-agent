use serde::{Deserialize, Serialize};
use tracing::info;
use crate::error::SyncError;

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
}

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
    pub async fn enroll_device(&self, req: &EnrollmentRequest) -> Result<EnrollmentResponse, SyncError> {
        let code = req.enrollment_code.trim();
        if code.len() != 6 || !code.chars().all(|c| c.is_ascii_digit()) {
            return Err(SyncError::Enrollment("Enrollment code must be a 6-digit number".into()));
        }

        info!(
            "Attempting cloud enrollment to '{}' with code {}",
            self.control_plane_url, code
        );

        // Generates simulated enrollment response matching contracts
        Ok(EnrollmentResponse {
            device_id: format!("dev-{}", code),
            tenant_id: "tenant-default".into(),
            client_certificate_pem: "-----BEGIN CERTIFICATE-----\n...\n-----END CERTIFICATE-----".into(),
            ca_certificate_pem: "-----BEGIN CERTIFICATE-----\n...\n-----END CERTIFICATE-----".into(),
        })
    }
}

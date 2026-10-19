use serde::{Deserialize, Serialize};
use crate::error::UpdateError;
use crate::verifier::UpdatePackageVerifier;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReleaseManifest {
    pub version: String,
    pub channel: String, // "stable", "beta"
    pub sha256: String,
    pub binary_url: String,
    pub signature_hex: String,
    pub min_supported_version: String,
}

pub struct StagedUpdateEngine {
    current_version: String,
}

impl StagedUpdateEngine {
    pub fn new(current_version: impl Into<String>) -> Self {
        Self {
            current_version: current_version.into(),
        }
    }

    /// Verifies manifest, channel, and package payload checksum.
    pub fn verify_release(&self, manifest: &ReleaseManifest, payload: &[u8]) -> Result<(), UpdateError> {
        if manifest.channel != "stable" && manifest.channel != "beta" {
            return Err(UpdateError::ManifestInvalid(format!(
                "Invalid channel: {}",
                manifest.channel
            )));
        }

        UpdatePackageVerifier::verify_checksum(payload, &manifest.sha256)?;
        Ok(())
    }

    /// Simulates staged atomic update: validates health check on new binary before swapping,
    /// triggering automated rollback if the new version fails its initial health check.
    pub fn stage_and_apply<F>(&self, manifest: &ReleaseManifest, payload: &[u8], health_check: F) -> Result<(), UpdateError>
    where
        F: FnOnce() -> bool,
    {
        self.verify_release(manifest, payload)?;

        // Run staged self-test
        if !health_check() {
            return Err(UpdateError::ManifestInvalid(
                "Health check failed on newly staged binary; executing automatic rollback to previous version".into(),
            ));
        }

        tracing::info!(
            "Atomic upgrade applied successfully: {} -> {}",
            self.current_version,
            manifest.version
        );
        Ok(())
    }
}

use std::path::PathBuf;
use tracing::info;

use crate::atomic_swap::WindowsAtomicUpdater;
use crate::error::UpdateError;
use crate::staged::{ReleaseManifest, StagedUpdateEngine, UpdateCheckResult};

/// OnlineUpdateClient coordinates checking for updates against the cloud server,
/// downloading payloads over secure transport, and staging them for atomic swap.
pub struct OnlineUpdateClient {
    server_base_url: String,
    channel: String,
    engine: StagedUpdateEngine,
}

impl OnlineUpdateClient {
    pub fn new(
        server_base_url: impl Into<String>,
        channel: impl Into<String>,
        engine: StagedUpdateEngine,
    ) -> Self {
        Self {
            server_base_url: server_base_url.into().trim_end_matches('/').to_string(),
            channel: channel.into(),
            engine,
        }
    }

    pub fn server_base_url(&self) -> &str {
        &self.server_base_url
    }

    pub fn channel(&self) -> &str {
        &self.channel
    }

    pub fn engine(&self) -> &StagedUpdateEngine {
        &self.engine
    }

    /// Generates standard HTTPS URL for fetching the release manifest.
    pub fn manifest_url(&self) -> String {
        format!(
            "{}/v1/channels/{}/manifest.json",
            self.server_base_url, self.channel
        )
    }

    /// Parse manifest JSON returned by server
    pub fn parse_manifest(&self, json_bytes: &[u8]) -> Result<ReleaseManifest, UpdateError> {
        serde_json::from_slice(json_bytes).map_err(|e| {
            UpdateError::ManifestInvalid(format!("Failed to parse release manifest: {e}"))
        })
    }

    /// Check if the parsed manifest contains an applicable update
    pub fn check_update(
        &self,
        manifest: &ReleaseManifest,
    ) -> Result<UpdateCheckResult, UpdateError> {
        self.engine.check_version(manifest)
    }

    /// Verifies downloaded payload against manifest and writes to staged directory.
    pub fn verify_and_stage(
        &self,
        manifest: &ReleaseManifest,
        payload: &[u8],
        target_binary_name: &str,
        updater: &WindowsAtomicUpdater,
    ) -> Result<PathBuf, UpdateError> {
        // 1. Verify digital signature & checksum
        self.engine.verify_release(manifest, payload)?;

        // 2. Stage binary
        let staged_path = updater.stage_binary(target_binary_name, payload, &manifest.sha256)?;
        info!(
            "Online update verified and staged at {:?} for binary '{}'",
            staged_path, target_binary_name
        );
        Ok(staged_path)
    }
}

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use tracing::info;

use crate::atomic_swap::WindowsAtomicUpdater;
use crate::error::UpdateError;
use crate::staged::{ReleaseManifest, StagedUpdateEngine, UpdateCheckResult};

/// OnlineUpdateClient coordinates checking for updates against the cloud server / edge worker,
/// downloading payloads over secure transport with resume support, and staging them for atomic swap.
pub struct OnlineUpdateClient {
    server_base_url: String,
    channel: String,
    engine: StagedUpdateEngine,
    http_client: reqwest::Client,
}

impl OnlineUpdateClient {
    pub fn new(
        server_base_url: impl Into<String>,
        channel: impl Into<String>,
        engine: StagedUpdateEngine,
    ) -> Self {
        let http_client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::limited(10))
            .build()
            .unwrap_or_default();

        Self {
            server_base_url: server_base_url.into().trim_end_matches('/').to_string(),
            channel: channel.into(),
            engine,
            http_client,
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

    /// Downloads a release binary with HTTP Range resumption and redirect support
    /// (e.g. from GitHub Releases redirecting to AWS/Fastly CDN).
    pub async fn download_resumable(
        &self,
        url: &str,
        target_path: &Path,
    ) -> Result<PathBuf, UpdateError> {
        let part_path = target_path.with_extension("part");

        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let existing_len = if part_path.exists() {
            fs::metadata(&part_path).map(|m| m.len()).unwrap_or(0)
        } else {
            0
        };

        let mut req = self.http_client.get(url);
        if existing_len > 0 {
            info!(
                "Resuming download of '{}' from byte offset {}",
                url, existing_len
            );
            req = req.header(reqwest::header::RANGE, format!("bytes={existing_len}-"));
        }

        let mut resp = req
            .send()
            .await
            .map_err(|e| UpdateError::Network(e.to_string()))?;
        let status = resp.status();

        let mut file = if status == reqwest::StatusCode::PARTIAL_CONTENT {
            // Append to existing partial file
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(&part_path)?
        } else if status == reqwest::StatusCode::OK {
            // Server ignored Range header or fresh download; start from beginning
            OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&part_path)?
        } else {
            return Err(UpdateError::Network(format!(
                "HTTP download failed with status {status} for {url}"
            )));
        };

        while let Some(chunk) = resp
            .chunk()
            .await
            .map_err(|e| UpdateError::Network(e.to_string()))?
        {
            file.write_all(&chunk)?;
        }
        file.flush()?;

        // Atomically rename .part file to final target
        if target_path.exists() {
            let _ = fs::remove_file(target_path);
        }
        fs::rename(&part_path, target_path)?;

        info!(
            "Download completed successfully. Saved to {:?}",
            target_path
        );
        Ok(target_path.to_path_buf())
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

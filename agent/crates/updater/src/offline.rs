use std::fs;
use std::path::{Path, PathBuf};
use tracing::info;

use crate::atomic_swap::WindowsAtomicUpdater;
use crate::error::UpdateError;
use crate::staged::{ReleaseManifest, StagedUpdateEngine, UpdateCheckResult};

#[derive(Clone, Debug)]
pub struct OfflineUpdateReport {
    pub previous_version: String,
    pub new_version: String,
    pub swapped_binary: String,
    pub backup_path: PathBuf,
}

pub struct OfflineUpdateEngine;

impl OfflineUpdateEngine {
    /// Inspect an offline package directory containing `manifest.json` and the target binary.
    /// Performs cryptographic verification without modifying the running system.
    pub fn check_offline_directory(
        package_dir: &Path,
        engine: &StagedUpdateEngine,
    ) -> Result<UpdateCheckResult, UpdateError> {
        let manifest_path = package_dir.join("manifest.json");
        if !manifest_path.exists() {
            return Err(UpdateError::PackageNotFound(format!(
                "manifest.json not found in {:?}",
                package_dir
            )));
        }

        let manifest_bytes = fs::read(&manifest_path)?;
        let manifest: ReleaseManifest = serde_json::from_slice(&manifest_bytes)
            .map_err(|e| UpdateError::ManifestInvalid(format!("Malformed manifest.json: {e}")))?;

        // First check version compatibility
        let check_res = engine.check_version(&manifest)?;
        match check_res {
            UpdateCheckResult::UpdateAvailable { .. } => {
                // Verify the binary payload in the package
                let binary_filename = Self::detect_binary_filename(package_dir, &manifest)?;
                let binary_path = package_dir.join(&binary_filename);
                let payload = fs::read(&binary_path)?;

                // Cryptographically verify signature and SHA-256
                engine.verify_release(&manifest, &payload)?;
                info!(
                    "Offline package verified OK: version {} (file: {:?})",
                    manifest.version, binary_path
                );
                Ok(check_res)
            }
            other => Ok(other),
        }
    }

    /// Applies an offline update package atomically:
    /// 1. Verifies manifest signature and binary SHA-256
    /// 2. Stages binary into update staging directory
    /// 3. Backs up current running binary
    /// 4. Performs atomic binary swap (with auto-rollback on failure)
    pub fn apply_offline_directory(
        package_dir: &Path,
        engine: &StagedUpdateEngine,
        updater: &WindowsAtomicUpdater,
    ) -> Result<OfflineUpdateReport, UpdateError> {
        let manifest_path = package_dir.join("manifest.json");
        if !manifest_path.exists() {
            return Err(UpdateError::PackageNotFound(format!(
                "manifest.json not found in {:?}",
                package_dir
            )));
        }

        let manifest_bytes = fs::read(&manifest_path)?;
        let manifest: ReleaseManifest = serde_json::from_slice(&manifest_bytes)
            .map_err(|e| UpdateError::ManifestInvalid(format!("Malformed manifest.json: {e}")))?;

        // Enforce version and signature validation
        let binary_filename = Self::detect_binary_filename(package_dir, &manifest)?;
        let binary_path = package_dir.join(&binary_filename);
        let payload = fs::read(&binary_path)?;

        engine.verify_release(&manifest, &payload)?;

        let prev_ver = updater.current_version().to_string();
        info!(
            "Applying offline atomic update: {} -> {}",
            prev_ver, manifest.version
        );

        // Stage binary
        updater.stage_binary(&binary_filename, &payload, &manifest.sha256)?;

        // Atomic swap
        updater.atomic_swap(&binary_filename)?;

        let backup_path = updater.backup_dir().join(&binary_filename);

        info!(
            "Offline update applied successfully! Previous version: {}, New version: {}",
            prev_ver, manifest.version
        );

        Ok(OfflineUpdateReport {
            previous_version: prev_ver,
            new_version: manifest.version,
            swapped_binary: binary_filename,
            backup_path,
        })
    }

    fn detect_binary_filename(
        package_dir: &Path,
        manifest: &ReleaseManifest,
    ) -> Result<String, UpdateError> {
        // 1. Try URL filename
        if let Some(name) = manifest.binary_url.split('/').next_back() {
            if !name.is_empty() && package_dir.join(name).exists() {
                return Ok(name.to_string());
            }
        }

        // 2. Try standard FamShield service binary name
        if package_dir.join("famshield-service.exe").exists() {
            return Ok("famshield-service.exe".to_string());
        }

        if package_dir.join("famshield-service.bin").exists() {
            return Ok("famshield-service.bin".to_string());
        }

        Err(UpdateError::PackageNotFound(format!(
            "No matching binary file found in {:?}",
            package_dir
        )))
    }
}

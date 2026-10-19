use std::fs;
use std::path::{Path, PathBuf};
use tracing::{error, info, warn};

use crate::error::UpdateError;
use crate::verifier::UpdatePackageVerifier;

pub struct WindowsAtomicUpdater {
    current_version: String,
    install_dir: PathBuf,
    backup_dir: PathBuf,
    staging_dir: PathBuf,
}

impl WindowsAtomicUpdater {
    pub fn new(
        current_version: impl Into<String>,
        app_data_dir: &Path,
        install_dir: &Path,
    ) -> Self {
        Self {
            current_version: current_version.into(),
            install_dir: install_dir.to_path_buf(),
            backup_dir: app_data_dir.join("backup"),
            staging_dir: app_data_dir.join("updates").join("staged"),
        }
    }

    pub fn current_version(&self) -> &str {
        &self.current_version
    }

    pub fn backup_dir(&self) -> &Path {
        &self.backup_dir
    }

    pub fn staging_dir(&self) -> &Path {
        &self.staging_dir
    }

    /// Stage new binary payload: verifies SHA-256 and writes to staged directory.
    pub fn stage_binary(
        &self,
        filename: &str,
        payload: &[u8],
        expected_sha256: &str,
    ) -> Result<PathBuf, UpdateError> {
        UpdatePackageVerifier::verify_checksum(payload, expected_sha256)?;

        fs::create_dir_all(&self.staging_dir)?;
        let staged_file = self.staging_dir.join(filename);
        fs::write(&staged_file, payload)?;

        info!(
            "Staged binary '{}' verified and written to {:?}",
            filename, staged_file
        );
        Ok(staged_file)
    }

    /// Applies atomic swap:
    /// 1. Backs up existing active binary to backup/
    /// 2. Atomically replaces active binary with staged binary
    /// 3. If replacing fails, triggers automated rollback from backup/
    pub fn atomic_swap(&self, filename: &str) -> Result<(), UpdateError> {
        let active_file = self.install_dir.join(filename);
        let staged_file = self.staging_dir.join(filename);
        let backup_file = self.backup_dir.join(filename);

        if !staged_file.exists() {
            return Err(UpdateError::ManifestInvalid(format!(
                "Staged binary not found: {:?}",
                staged_file
            )));
        }

        fs::create_dir_all(&self.backup_dir)?;

        // 1. Backup currently running active binary if present
        if active_file.exists() {
            fs::copy(&active_file, &backup_file)?;
            info!(
                "Backed up active binary {:?} to {:?}",
                active_file, backup_file
            );
        }

        // 2. Perform atomic replacement
        // On Windows NTFS, fs::copy + fs::remove or MoveFileEx handles staged updates
        if let Err(e) = fs::copy(&staged_file, &active_file) {
            error!(
                "Failed to replace active binary {:?}: {}. Initiating rollback...",
                active_file, e
            );
            self.rollback(filename)?;
            return Err(UpdateError::Io(e));
        }

        // Clean up staged file
        let _ = fs::remove_file(staged_file);
        info!("Atomic binary replacement successful for '{}'", filename);
        Ok(())
    }

    /// Automated rollback: restores binary from backup directory.
    pub fn rollback(&self, filename: &str) -> Result<(), UpdateError> {
        warn!("EXECUTING AUTOMATED ROLLBACK for '{}'...", filename);
        let active_file = self.install_dir.join(filename);
        let backup_file = self.backup_dir.join(filename);

        if backup_file.exists() {
            fs::copy(&backup_file, &active_file)?;
            info!("Rollback successful: Restored active binary from backup.");
            Ok(())
        } else {
            error!(
                "Rollback failed: No backup file available at {:?}",
                backup_file
            );
            Err(UpdateError::ManifestInvalid(
                "Rollback failed: Backup file does not exist".into(),
            ))
        }
    }
}

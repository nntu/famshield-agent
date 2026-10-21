use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{error, info, warn};

use crate::error::UpdateError;
use crate::verifier::UpdatePackageVerifier;

/// Status returned during service startup health verification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WatchdogStatus {
    /// Normal operation, no pending update verification
    Normal,
    /// Update was recently applied; current boot is under 60s observation window
    PendingVerification {
        target_version: String,
        boot_attempts: u32,
    },
    /// Detected previous crash before verification; service was automatically rolled back
    RolledBack {
        failed_version: String,
        restored_version: String,
    },
    /// The update was successfully confirmed healthy
    Verified,
}

/// Persistent state stored in watchdog.json to track update survival across restarts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateWatchdogState {
    pub target_binary: String,
    pub target_version: String,
    pub previous_version: String,
    pub timestamp_secs: u64,
    pub boot_attempts: u32,
    pub verified: bool,
}

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

    pub fn install_dir(&self) -> &Path {
        &self.install_dir
    }

    pub fn backup_dir(&self) -> &Path {
        &self.backup_dir
    }

    pub fn staging_dir(&self) -> &Path {
        &self.staging_dir
    }

    pub fn watchdog_file_path(&self) -> PathBuf {
        self.staging_dir.join("watchdog.json")
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

    /// Records watchdog state before service restart so we know an update was applied
    pub fn record_pending_update(
        &self,
        filename: &str,
        new_version: &str,
    ) -> Result<(), UpdateError> {
        fs::create_dir_all(&self.staging_dir)?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let state = UpdateWatchdogState {
            target_binary: filename.to_string(),
            target_version: new_version.to_string(),
            previous_version: self.current_version.clone(),
            timestamp_secs: now,
            boot_attempts: 0,
            verified: false,
        };

        let json = serde_json::to_string_pretty(&state).map_err(|e| {
            UpdateError::ManifestInvalid(format!("Failed to serialize watchdog state: {e}"))
        })?;
        fs::write(self.watchdog_file_path(), json)?;
        info!(
            "Watchdog state recorded for pending update to v{} (target: {})",
            new_version, filename
        );
        Ok(())
    }

    /// Loads active watchdog state if present
    pub fn load_watchdog_state(&self) -> Option<UpdateWatchdogState> {
        let path = self.watchdog_file_path();
        if !path.exists() {
            return None;
        }
        let data = fs::read_to_string(&path).ok()?;
        serde_json::from_str(&data).ok()
    }

    /// Called at Service startup: checks if the previous boot was an unverified update that crashed.
    /// If boot_attempts > 1, automated rollback is triggered to protect against bricking.
    pub fn check_and_recover_health(&self) -> Result<WatchdogStatus, UpdateError> {
        let mut state = match self.load_watchdog_state() {
            Some(s) => s,
            None => return Ok(WatchdogStatus::Normal),
        };

        if state.verified {
            let _ = fs::remove_file(self.watchdog_file_path());
            return Ok(WatchdogStatus::Verified);
        }

        state.boot_attempts += 1;

        // If boot_attempts > 1, the newly updated binary crashed before calling confirm_update_healthy!
        if state.boot_attempts > 1 {
            error!(
                "CRITICAL: Watchdog detected boot failure after updating to v{}! Automated rollback initiating...",
                state.target_version
            );

            // Execute rollback to previous binary
            self.rollback(&state.target_binary)?;

            let failed_ver = state.target_version.clone();
            let restored_ver = state.previous_version.clone();

            // Clear watchdog state after rollback
            let _ = fs::remove_file(self.watchdog_file_path());

            return Ok(WatchdogStatus::RolledBack {
                failed_version: failed_ver,
                restored_version: restored_ver,
            });
        }

        // First boot attempt: save updated attempt count and allow service to start observation window
        if let Ok(json) = serde_json::to_string_pretty(&state) {
            let _ = fs::write(self.watchdog_file_path(), json);
        }

        info!(
            "Watchdog: Boot attempt {} for new version v{}. Observation window active.",
            state.boot_attempts, state.target_version
        );

        Ok(WatchdogStatus::PendingVerification {
            target_version: state.target_version,
            boot_attempts: state.boot_attempts,
        })
    }

    /// Called once the Service has verified normal operation (e.g. WFP/DNS up, 60s passed without crash)
    pub fn confirm_update_healthy(&self) -> Result<(), UpdateError> {
        let path = self.watchdog_file_path();
        if path.exists() {
            let _ = fs::remove_file(&path);
            info!("Update verified as healthy! Watchdog state cleared.");
        }

        // Clean up temporary .old binary in install dir
        if let Ok(entries) = fs::read_dir(&self.install_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if let Some(ext) = p.extension() {
                    if ext == "old" {
                        let _ = fs::remove_file(&p);
                    }
                }
            }
        }
        Ok(())
    }

    /// Applies atomic swap:
    /// 1. Backs up existing active binary to backup/
    /// 2. Renames active binary to .old (Windows allows renaming running executables)
    /// 3. Copies staged binary into place
    /// 4. If replacing fails, triggers automated rollback
    pub fn atomic_swap(&self, filename: &str) -> Result<(), UpdateError> {
        let active_file = self.install_dir.join(filename);
        let staged_file = self.staging_dir.join(filename);
        let backup_file = self.backup_dir.join(filename);
        let old_file = self.install_dir.join(format!("{filename}.old"));

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

            // Windows NTFS allows renaming an executing binary even when locked
            if old_file.exists() {
                let _ = fs::remove_file(&old_file);
            }
            if let Err(e) = fs::rename(&active_file, &old_file) {
                warn!("Could not rename active file to .old: {e}. Trying direct copy.");
            }
        }

        // 2. Perform replacement
        if let Err(e) = fs::copy(&staged_file, &active_file) {
            error!(
                "Failed to replace active binary {:?}: {}. Initiating rollback...",
                active_file, e
            );
            // Restore from old_file if present
            if old_file.exists() {
                let _ = fs::rename(&old_file, &active_file);
            }
            self.rollback(filename)?;
            return Err(UpdateError::Io(e));
        }

        // Clean up staged file
        let _ = fs::remove_file(&staged_file);
        info!("Atomic binary replacement successful for '{}'", filename);
        Ok(())
    }

    /// Automated rollback: restores binary from backup directory or .old file.
    pub fn rollback(&self, filename: &str) -> Result<(), UpdateError> {
        warn!("EXECUTING AUTOMATED ROLLBACK for '{}'...", filename);
        let active_file = self.install_dir.join(filename);
        let backup_file = self.backup_dir.join(filename);
        let old_file = self.install_dir.join(format!("{filename}.old"));

        if old_file.exists() {
            if let Err(e) = fs::rename(&old_file, &active_file) {
                warn!("Could not rename .old back to active: {e}. Falling back to backup copy.");
                if backup_file.exists() {
                    fs::copy(&backup_file, &active_file)?;
                } else {
                    return Err(UpdateError::Io(e));
                }
            }
            info!("Rollback successful: Restored active binary from .old.");
            Ok(())
        } else if backup_file.exists() {
            fs::copy(&backup_file, &active_file)?;
            info!("Rollback successful: Restored active binary from backup directory.");
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

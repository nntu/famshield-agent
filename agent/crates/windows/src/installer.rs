use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::{info, warn};
use crate::error::WindowsError;

/// Details of a software installation attempt
#[derive(Clone, Debug)]
pub struct InstallationRequest {
    pub request_id: String,
    pub process_id: u32,
    pub executable_path: String,
    pub file_hash_sha256: String,
    pub publisher: Option<String>,
}

/// Installation Interceptor & Approval Manager
pub struct InstallerInterceptor {
    /// Tracks pending installation requests awaiting server or parent approval
    pending: Arc<Mutex<HashMap<String, InstallationRequest>>>,
    /// Approved hashes with validity (or permanent)
    approved_hashes: Arc<Mutex<HashMap<String, u64>>>,
}

impl Default for InstallerInterceptor {
    fn default() -> Self {
        Self::new()
    }
}

impl InstallerInterceptor {
    pub fn new() -> Self {
        Self {
            pending: Arc::new(Mutex::new(HashMap::new())),
            approved_hashes: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Check if an executable matches typical installer/setup patterns
    pub fn is_installer_binary(path_or_name: &str) -> bool {
        let lower = path_or_name.to_ascii_lowercase();
        let file_name = lower.rsplit(&['/', '\\'][..]).next().unwrap_or(&lower);

        let installer_keywords = [
            "setup", "install", "installer", "update", "patch", 
            "msiexec", "unins", "config", "deploy"
        ];

        let has_keyword = installer_keywords.iter().any(|&kw| file_name.contains(kw));
        let is_executable_ext = lower.ends_with(".exe") || lower.ends_with(".msi");

        has_keyword && is_executable_ext
    }

    /// Check if a binary hash has already been approved by server or parent
    pub fn is_hash_approved(&self, sha256_hash: &str) -> bool {
        let guard = self.approved_hashes.lock().unwrap();
        guard.contains_key(sha256_hash)
    }

    /// Submit a pending installation request for approval
    pub fn submit_request(&self, req: InstallationRequest) {
        info!(
            "Intercepted installer execution: PID {} ('{}', hash: {}). Suspending and awaiting server approval...",
            req.process_id, req.executable_path, req.file_hash_sha256
        );
        let mut guard = self.pending.lock().unwrap();
        guard.insert(req.request_id.clone(), req);
    }

    /// Approve an installation request by request_id or file_hash
    pub fn approve_installation(&self, request_id_or_hash: &str) -> Result<Option<InstallationRequest>, WindowsError> {
        let mut pending_guard = self.pending.lock().unwrap();
        let mut approved_guard = self.approved_hashes.lock().unwrap();

        // Check if matching by request_id
        if let Some(req) = pending_guard.remove(request_id_or_hash) {
            approved_guard.insert(req.file_hash_sha256.clone(), 0);
            info!("Installation request '{}' approved! Resuming process PID {}", req.request_id, req.process_id);
            return Ok(Some(req));
        }

        // Check if matching by file_hash
        let mut matched_key = None;
        for (k, v) in pending_guard.iter() {
            if v.file_hash_sha256 == request_id_or_hash {
                matched_key = Some(k.clone());
                break;
            }
        }

        if let Some(k) = matched_key {
            let req = pending_guard.remove(&k).unwrap();
            approved_guard.insert(req.file_hash_sha256.clone(), 0);
            info!("Installation for hash '{}' approved! Resuming process PID {}", request_id_or_hash, req.process_id);
            return Ok(Some(req));
        }

        // Otherwise just record the hash as pre-approved
        approved_guard.insert(request_id_or_hash.to_string(), 0);
        info!("Hash '{}' added to approved installer whitelist", request_id_or_hash);
        Ok(None)
    }

    /// Deny and reject an installation attempt
    pub fn deny_installation(&self, request_id_or_hash: &str) -> Result<Option<InstallationRequest>, WindowsError> {
        let mut pending_guard = self.pending.lock().unwrap();

        if let Some(req) = pending_guard.remove(request_id_or_hash) {
            warn!("Installation request '{}' denied! Terminating process PID {}", req.request_id, req.process_id);
            return Ok(Some(req));
        }

        let mut matched_key = None;
        for (k, v) in pending_guard.iter() {
            if v.file_hash_sha256 == request_id_or_hash {
                matched_key = Some(k.clone());
                break;
            }
        }

        if let Some(k) = matched_key {
            let req = pending_guard.remove(&k).unwrap();
            warn!("Installation for hash '{}' denied! Terminating process PID {}", request_id_or_hash, req.process_id);
            return Ok(Some(req));
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_installer_binary() {
        assert!(InstallerInterceptor::is_installer_binary("C:\\Users\\Kid\\Downloads\\RobloxPlayerSetup.exe"));
        assert!(InstallerInterceptor::is_installer_binary("SteamSetup.exe"));
        assert!(InstallerInterceptor::is_installer_binary("installer_v2.msi"));
        assert!(InstallerInterceptor::is_installer_binary("game_patch.exe"));
        assert!(!InstallerInterceptor::is_installer_binary("notepad.exe"));
        assert!(!InstallerInterceptor::is_installer_binary("chrome.exe"));
    }

    #[test]
    fn test_approve_deny_flow() {
        let interceptor = InstallerInterceptor::new();
        let req = InstallationRequest {
            request_id: "req-123".into(),
            process_id: 4567,
            executable_path: "C:\\Temp\\RobloxSetup.exe".into(),
            file_hash_sha256: "a1b2c3d4e5f6".into(),
            publisher: Some("Roblox Corporation".into()),
        };

        interceptor.submit_request(req);
        assert!(!interceptor.is_hash_approved("a1b2c3d4e5f6"));

        // Approve
        let res = interceptor.approve_installation("req-123").unwrap();
        assert!(res.is_some());
        assert_eq!(res.unwrap().process_id, 4567);
        assert!(interceptor.is_hash_approved("a1b2c3d4e5f6"));
    }
}

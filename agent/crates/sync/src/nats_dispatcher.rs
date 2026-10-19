use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, warn};

use crate::error::SyncError;
use famshield_dns::blocklist::BlocklistStore;
use famshield_local_admin::MasterPinAuthenticator;
use famshield_storage::StorageEngine;
use famshield_windows::{InstallerInterceptor, SessionManager};

/// Command payload pushed from Server via NATS JetStream: `devices.{device_id}.commands`
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RemoteCommand {
    pub command_id: String,
    pub device_id: String,
    pub command_type: String, // "lock_session", "update_blacklist", "reset_pin", "update_policy", "approve_install", "deny_install"
    #[serde(default)]
    pub timestamp: i64, // Unix epoch seconds
    #[serde(default)]
    pub payload: serde_json::Value,
    #[serde(default)]
    pub signature: Option<String>, // HMAC-SHA256 signature
}

/// Canonical HMAC-SHA256 signature calculation over RemoteCommand fields:
/// `command_id:device_id:command_type:timestamp:payload_json`
pub fn compute_command_hmac(secret_key: &str, cmd: &RemoteCommand) -> String {
    use sha2::{Digest, Sha256};
    let payload_str = cmd.payload.to_string();
    let message = format!(
        "{}:{}:{}:{}:{}",
        cmd.command_id, cmd.device_id, cmd.command_type, cmd.timestamp, payload_str
    );

    let block_size = 64;
    let mut key = secret_key.as_bytes().to_vec();
    if key.len() > block_size {
        let mut hasher = Sha256::new();
        hasher.update(&key);
        key = hasher.finalize().to_vec();
    }
    if key.len() < block_size {
        key.resize(block_size, 0);
    }

    let mut o_key_pad = vec![0x5cu8; block_size];
    let mut i_key_pad = vec![0x36u8; block_size];
    for i in 0..block_size {
        o_key_pad[i] ^= key[i];
        i_key_pad[i] ^= key[i];
    }

    let mut inner = Sha256::new();
    inner.update(&i_key_pad);
    inner.update(message.as_bytes());
    let inner_hash = inner.finalize();

    let mut outer = Sha256::new();
    outer.update(&o_key_pad);
    outer.update(inner_hash);
    let result = outer.finalize();

    format!("{:x}", result)
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

/// NatsDispatcher orchestrates real-time top-down commands received from NATS
/// and bottom-up event pushes from local outbox.
pub struct NatsDispatcher {
    device_id: String,
    storage: StorageEngine,
    blocklist: Arc<BlocklistStore>,
    installer: Arc<InstallerInterceptor>,
    signing_key: Option<String>,
}

impl NatsDispatcher {
    pub fn new(
        device_id: impl Into<String>,
        storage: StorageEngine,
        blocklist: Arc<BlocklistStore>,
        installer: Arc<InstallerInterceptor>,
    ) -> Self {
        Self {
            device_id: device_id.into(),
            storage,
            blocklist,
            installer,
            signing_key: None,
        }
    }

    pub fn with_signing_key(mut self, signing_key: impl Into<String>) -> Self {
        self.signing_key = Some(signing_key.into());
        self
    }

    pub fn set_signing_key(&mut self, signing_key: Option<String>) {
        self.signing_key = signing_key;
    }

    pub fn device_id(&self) -> &str {
        &self.device_id
    }

    /// Subject topic for receiving commands from Server
    pub fn command_subject(&self) -> String {
        format!("devices.{}.commands", self.device_id)
    }

    /// Subject topic for sending events/telemetry up to Server JetStream
    pub fn event_subject(&self) -> String {
        format!("devices.{}.events", self.device_id)
    }

    /// Subject topic for sending heartbeat pings
    pub fn heartbeat_subject(&self) -> String {
        format!("devices.{}.heartbeat", self.device_id)
    }

    /// Build a standardized telemetry payload for a process lifecycle event
    pub fn build_process_event(
        &self,
        event: &famshield_windows::ProcessLifecycleEvent,
    ) -> serde_json::Value {
        serde_json::json!({
            "event_id": event.event_id,
            "device_id": self.device_id,
            "event_type": event.event_type,
            "occurred_at": event.occurred_at,
            "payload": {
                "pid": event.pid,
                "ppid": event.ppid,
                "process_name": event.process_name,
            }
        })
    }

    /// Serialize a batch of process events ready for pushing to Server via NATS JetStream
    pub fn format_process_events_batch(
        &self,
        events: &[famshield_windows::ProcessLifecycleEvent],
    ) -> Result<String, SyncError> {
        let serialized_events: Vec<serde_json::Value> =
            events.iter().map(|e| self.build_process_event(e)).collect();

        let batch = serde_json::json!({
            "batch_id": format!("batch-proc-{}", chrono::Utc::now().timestamp_millis()),
            "device_id": self.device_id,
            "count": serialized_events.len(),
            "events": serialized_events,
        });

        serde_json::to_string(&batch).map_err(SyncError::Serialization)
    }

    /// Serialize a batch of unsynced error logs to be pushed to Server via NATS JetStream
    pub fn format_error_logs_batch(
        &self,
        errors: &[famshield_storage::ErrorLogRecord],
    ) -> Result<String, SyncError> {
        let serialized_errors: Vec<serde_json::Value> = errors
            .iter()
            .map(|e| {
                serde_json::json!({
                    "error_id": e.error_id,
                    "device_id": self.device_id,
                    "module": e.module,
                    "severity": e.severity,
                    "message": e.message,
                    "stack_trace": e.stack_trace,
                    "occurrence_count": e.occurrence_count,
                    "first_seen_at": e.first_seen_at.to_rfc3339(),
                    "last_seen_at": e.last_seen_at.to_rfc3339(),
                })
            })
            .collect();

        let batch = serde_json::json!({
            "batch_id": format!("batch-err-{}", chrono::Utc::now().timestamp_millis()),
            "device_id": self.device_id,
            "count": serialized_errors.len(),
            "errors": serialized_errors,
        });

        serde_json::to_string(&batch).map_err(SyncError::Serialization)
    }

    /// Verify command authenticity, integrity, and anti-replay window
    pub fn verify_command(&self, cmd: &RemoteCommand) -> Result<(), SyncError> {
        // 1. Verify target device
        if cmd.device_id != self.device_id {
            return Err(SyncError::Authentication(format!(
                "Device ID mismatch: expected '{}', got '{}'",
                self.device_id, cmd.device_id
            )));
        }

        // 2. If signing key is configured (Managed Cloud Mode), enforce cryptographic checks
        if let Some(ref key) = self.signing_key {
            let now = chrono::Utc::now().timestamp();
            if cmd.timestamp <= 0 {
                return Err(SyncError::Authentication(
                    "Missing or invalid command timestamp".into(),
                ));
            }

            let drift = (now - cmd.timestamp).abs();
            if drift > 300 {
                return Err(SyncError::Authentication(format!(
                    "Command timestamp expired or clock drift exceeded (drift: {}s, max: 300s)",
                    drift
                )));
            }

            let sig = cmd.signature.as_deref().ok_or_else(|| {
                SyncError::Authentication(
                    "Missing required HMAC signature for remote command".into(),
                )
            })?;

            let expected_sig = compute_command_hmac(key, cmd);
            if !constant_time_eq(sig.as_bytes(), expected_sig.as_bytes()) {
                return Err(SyncError::Authentication(
                    "Invalid HMAC command signature".into(),
                ));
            }
        }

        Ok(())
    }

    /// Execute incoming command immediately without polling or latency
    pub fn dispatch_command(&self, cmd: &RemoteCommand) -> Result<(), SyncError> {
        self.verify_command(cmd)?;

        info!(
            "Processing remote command '{}' (ID: {}) for device '{}'",
            cmd.command_type, cmd.command_id, self.device_id
        );

        match cmd.command_type.as_str() {
            "lock_session" => {
                info!("Executing immediate session lock via LockWorkStation API...");
                SessionManager::lock_workstation().map_err(|e| {
                    SyncError::Command(format!("Failed to lock workstation: {}", e))
                })?;
                info!("Session locked successfully.");
                Ok(())
            }

            "update_blacklist" => {
                // Extract list of domain strings from payload
                let domains: Vec<String> =
                    if let Some(arr) = cmd.payload.get("domains").and_then(|v| v.as_array()) {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    } else if let Some(arr) = cmd.payload.as_array() {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    } else {
                        return Err(SyncError::Command(
                            "Payload must contain 'domains' array or be a string array".into(),
                        ));
                    };

                info!(
                    "Atomically hot-swapping DNS blocklist with {} domains...",
                    domains.len()
                );
                self.blocklist.replace_blocklist(&domains);
                info!("DNS blocklist successfully updated with 0 downtime.");
                Ok(())
            }

            "reset_pin" => {
                let new_pin = cmd
                    .payload
                    .get("new_pin")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| SyncError::Command("Missing 'new_pin' in payload".into()))?;

                info!("Executing remote cloud Master PIN reset...");
                MasterPinAuthenticator::reset_pin_elevated(&self.storage, new_pin)
                    .map_err(|e| SyncError::Command(format!("Failed to reset PIN: {}", e)))?;
                info!("Master PIN successfully reset via cloud command.");
                Ok(())
            }

            "update_policy" => {
                info!("Policy update command received: {:?}", cmd.payload);
                // Policy update payload can be parsed and saved to storage
                if let Some(policy_obj) = cmd.payload.get("policy") {
                    let policy_str = policy_obj.to_string();
                    if let Ok(policy) = serde_json::from_str::<famshield_core::Policy>(&policy_str)
                    {
                        self.storage.save_policy(&policy, &policy_str)?;
                        info!("Policy v{} applied and persisted.", policy.version);
                    }
                }
                Ok(())
            }

            "approve_install" => {
                let id_or_hash = cmd
                    .payload
                    .get("request_id")
                    .or_else(|| cmd.payload.get("file_hash"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        SyncError::Command("Missing 'request_id' or 'file_hash'".into())
                    })?;

                info!("Server approved installation for: {}", id_or_hash);
                self.installer
                    .approve_installation(id_or_hash)
                    .map_err(|e| SyncError::Command(format!("Failed to approve install: {}", e)))?;
                Ok(())
            }

            "deny_install" => {
                let id_or_hash = cmd
                    .payload
                    .get("request_id")
                    .or_else(|| cmd.payload.get("file_hash"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        SyncError::Command("Missing 'request_id' or 'file_hash'".into())
                    })?;

                info!("Server denied installation for: {}", id_or_hash);
                self.installer
                    .deny_installation(id_or_hash)
                    .map_err(|e| SyncError::Command(format!("Failed to deny install: {}", e)))?;
                Ok(())
            }

            unknown => {
                warn!("Received unknown remote command type: '{}'", unknown);
                Err(SyncError::Command(format!(
                    "Unknown command type: {}",
                    unknown
                )))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nats_command_subjects() {
        let storage = StorageEngine::in_memory().unwrap();
        let blocklist = Arc::new(BlocklistStore::new());
        let installer = Arc::new(InstallerInterceptor::new());
        let dispatcher = NatsDispatcher::new("dev-999888", storage, blocklist, installer);

        assert_eq!(dispatcher.command_subject(), "devices.dev-999888.commands");
        assert_eq!(dispatcher.event_subject(), "devices.dev-999888.events");
        assert_eq!(
            dispatcher.heartbeat_subject(),
            "devices.dev-999888.heartbeat"
        );
    }

    #[test]
    fn test_dispatch_update_blacklist() {
        let storage = StorageEngine::in_memory().unwrap();
        let blocklist = Arc::new(BlocklistStore::new());
        let installer = Arc::new(InstallerInterceptor::new());
        let dispatcher = NatsDispatcher::new("dev-123456", storage, blocklist.clone(), installer);

        assert!(!blocklist.is_blocked("tiktok.com"));

        let cmd = RemoteCommand {
            command_id: "cmd-001".into(),
            device_id: "dev-123456".into(),
            command_type: "update_blacklist".into(),
            timestamp: 0,
            payload: serde_json::json!({
                "domains": ["tiktok.com", "facebook.com", "youtube.com"]
            }),
            signature: None,
        };

        dispatcher.dispatch_command(&cmd).unwrap();

        assert!(blocklist.is_blocked("tiktok.com"));
        assert!(blocklist.is_blocked("sub.tiktok.com"));
        assert!(blocklist.is_blocked("facebook.com"));
        assert!(!blocklist.is_blocked("google.com"));
    }

    #[test]
    fn test_dispatch_reset_pin() {
        let storage = StorageEngine::in_memory().unwrap();
        let blocklist = Arc::new(BlocklistStore::new());
        let installer = Arc::new(InstallerInterceptor::new());
        let dispatcher = NatsDispatcher::new("dev-123456", storage.clone(), blocklist, installer);

        let cmd = RemoteCommand {
            command_id: "cmd-002".into(),
            device_id: "dev-123456".into(),
            command_type: "reset_pin".into(),
            timestamp: 0,
            payload: serde_json::json!({
                "new_pin": "9876"
            }),
            signature: None,
        };

        dispatcher.dispatch_command(&cmd).unwrap();
        assert!(MasterPinAuthenticator::verify_storage_pin(&storage, "9876").unwrap());
        assert!(!MasterPinAuthenticator::verify_storage_pin(&storage, "1234").unwrap());
    }

    #[test]
    fn test_dispatch_lock_session() {
        let storage = StorageEngine::in_memory().unwrap();
        let blocklist = Arc::new(BlocklistStore::new());
        let installer = Arc::new(InstallerInterceptor::new());
        let dispatcher = NatsDispatcher::new("dev-123456", storage, blocklist, installer);

        let cmd = RemoteCommand {
            command_id: "cmd-003".into(),
            device_id: "dev-123456".into(),
            command_type: "lock_session".into(),
            timestamp: 0,
            payload: serde_json::json!({}),
            signature: None,
        };

        let res = dispatcher.dispatch_command(&cmd);
        assert!(res.is_ok());
    }

    #[test]
    fn test_dispatch_approve_deny_install() {
        use famshield_windows::InstallationRequest;
        let storage = StorageEngine::in_memory().unwrap();
        let blocklist = Arc::new(BlocklistStore::new());
        let installer = Arc::new(InstallerInterceptor::new());
        let dispatcher = NatsDispatcher::new("dev-123456", storage, blocklist, installer.clone());

        installer.submit_request(InstallationRequest {
            request_id: "req-app-01".into(),
            process_id: 9999,
            executable_path: "C:\\Downloads\\GameSetup.exe".into(),
            file_hash_sha256: "hash-game-123".into(),
            publisher: Some("Epic Games".into()),
        });

        assert!(!installer.is_hash_approved("hash-game-123"));

        // Approve command from server
        let approve_cmd = RemoteCommand {
            command_id: "cmd-004".into(),
            device_id: "dev-123456".into(),
            command_type: "approve_install".into(),
            timestamp: 0,
            payload: serde_json::json!({ "request_id": "req-app-01" }),
            signature: None,
        };
        dispatcher.dispatch_command(&approve_cmd).unwrap();
        assert!(installer.is_hash_approved("hash-game-123"));
    }

    #[test]
    fn test_command_hmac_verification_success() {
        let storage = StorageEngine::in_memory().unwrap();
        let blocklist = Arc::new(BlocklistStore::new());
        let installer = Arc::new(InstallerInterceptor::new());
        let signing_key = "secure-command-signing-key-998877";

        let dispatcher =
            NatsDispatcher::new("dev-verified-01", storage, blocklist.clone(), installer)
                .with_signing_key(signing_key);

        let mut cmd = RemoteCommand {
            command_id: "cmd-sec-101".into(),
            device_id: "dev-verified-01".into(),
            command_type: "update_blacklist".into(),
            timestamp: chrono::Utc::now().timestamp(),
            payload: serde_json::json!({ "domains": ["evil-site.com"] }),
            signature: None,
        };

        // Compute valid signature
        let sig = compute_command_hmac(signing_key, &cmd);
        cmd.signature = Some(sig);

        // Verification should succeed and command executes
        assert!(dispatcher.dispatch_command(&cmd).is_ok());
        assert!(blocklist.is_blocked("evil-site.com"));
    }

    #[test]
    fn test_command_hmac_tampered_payload_rejected() {
        let storage = StorageEngine::in_memory().unwrap();
        let blocklist = Arc::new(BlocklistStore::new());
        let installer = Arc::new(InstallerInterceptor::new());
        let signing_key = "secret-key-123";

        let dispatcher = NatsDispatcher::new("dev-tamper-01", storage, blocklist, installer)
            .with_signing_key(signing_key);

        let mut cmd = RemoteCommand {
            command_id: "cmd-sec-102".into(),
            device_id: "dev-tamper-01".into(),
            command_type: "lock_session".into(),
            timestamp: chrono::Utc::now().timestamp(),
            payload: serde_json::json!({ "reason": "bedtime" }),
            signature: None,
        };

        let sig = compute_command_hmac(signing_key, &cmd);
        cmd.signature = Some(sig);

        // Tamper payload after signing
        cmd.payload = serde_json::json!({ "reason": "tampered_evil_instruction" });

        let res = dispatcher.dispatch_command(&cmd);
        assert!(res.is_err());
        match res.unwrap_err() {
            SyncError::Authentication(msg) => {
                assert!(msg.contains("Invalid HMAC command signature"))
            }
            other => panic!("Expected Authentication error, got: {:?}", other),
        }
    }

    #[test]
    fn test_command_replay_window_expired() {
        let storage = StorageEngine::in_memory().unwrap();
        let blocklist = Arc::new(BlocklistStore::new());
        let installer = Arc::new(InstallerInterceptor::new());
        let signing_key = "secret-key-123";

        let dispatcher = NatsDispatcher::new("dev-replay-01", storage, blocklist, installer)
            .with_signing_key(signing_key);

        // Timestamp from 10 minutes ago (> 300s window)
        let old_timestamp = chrono::Utc::now().timestamp() - 600;
        let mut cmd = RemoteCommand {
            command_id: "cmd-sec-103".into(),
            device_id: "dev-replay-01".into(),
            command_type: "lock_session".into(),
            timestamp: old_timestamp,
            payload: serde_json::json!({}),
            signature: None,
        };

        let sig = compute_command_hmac(signing_key, &cmd);
        cmd.signature = Some(sig);

        let res = dispatcher.dispatch_command(&cmd);
        assert!(res.is_err());
        match res.unwrap_err() {
            SyncError::Authentication(msg) => assert!(msg.contains("drift exceeded")),
            other => panic!("Expected Authentication error, got: {:?}", other),
        }
    }

    #[test]
    fn test_command_device_mismatch() {
        let storage = StorageEngine::in_memory().unwrap();
        let blocklist = Arc::new(BlocklistStore::new());
        let installer = Arc::new(InstallerInterceptor::new());

        let dispatcher = NatsDispatcher::new("dev-mine", storage, blocklist, installer);

        let cmd = RemoteCommand {
            command_id: "cmd-sec-104".into(),
            device_id: "dev-other".into(),
            command_type: "lock_session".into(),
            timestamp: 0,
            payload: serde_json::json!({}),
            signature: None,
        };

        let res = dispatcher.dispatch_command(&cmd);
        assert!(res.is_err());
        match res.unwrap_err() {
            SyncError::Authentication(msg) => assert!(msg.contains("Device ID mismatch")),
            other => panic!("Expected Authentication error, got: {:?}", other),
        }
    }

    #[test]
    fn test_format_process_events_batch() {
        use famshield_windows::ProcessLifecycleEvent;
        let storage = StorageEngine::in_memory().unwrap();
        let blocklist = Arc::new(BlocklistStore::new());
        let installer = Arc::new(InstallerInterceptor::new());
        let dispatcher = NatsDispatcher::new("dev-555666", storage, blocklist, installer);

        let events = vec![
            ProcessLifecycleEvent {
                event_id: "evt-proc-01".into(),
                event_type: "process_started".into(),
                pid: 5012,
                ppid: 1000,
                process_name: "Discord.exe".into(),
                occurred_at: "2026-10-19T13:00:00Z".into(),
            },
            ProcessLifecycleEvent {
                event_id: "evt-proc-02".into(),
                event_type: "process_terminated".into(),
                pid: 5012,
                ppid: 1000,
                process_name: "Discord.exe".into(),
                occurred_at: "2026-10-19T13:30:00Z".into(),
            },
        ];

        let json_str = dispatcher.format_process_events_batch(&events).unwrap();
        assert!(json_str.contains("Discord.exe"));
        assert!(json_str.contains("process_started"));
        assert!(json_str.contains("process_terminated"));
        assert!(json_str.contains("dev-555666"));
    }

    #[test]
    fn test_format_error_logs_batch() {
        use famshield_storage::ErrorLogRecord;
        let storage = StorageEngine::in_memory().unwrap();
        let blocklist = Arc::new(BlocklistStore::new());
        let installer = Arc::new(InstallerInterceptor::new());
        let dispatcher = NatsDispatcher::new("dev-555666", storage, blocklist, installer);

        let now = chrono::Utc::now();
        let errors = vec![ErrorLogRecord {
            error_id: "err-dns-01".into(),
            module: "dns_proxy".into(),
            severity: "ERROR".into(),
            message: "Upstream DNS timeout: 1.1.1.1:53".into(),
            stack_trace: Some("at dns::proxy::resolve".into()),
            occurrence_count: 5,
            first_seen_at: now,
            last_seen_at: now,
            synced: false,
        }];

        let json_str = dispatcher.format_error_logs_batch(&errors).unwrap();
        assert!(json_str.contains("err-dns-01"));
        assert!(json_str.contains("Upstream DNS timeout"));
        assert!(json_str.contains("dns_proxy"));
    }
}

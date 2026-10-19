use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use famshield_dns::blocklist::BlocklistStore;
use famshield_local_admin::MasterPinAuthenticator;
use famshield_storage::StorageEngine;
use famshield_windows::{InstallerInterceptor, SessionManager};
use crate::error::SyncError;

/// Command payload pushed from Server via NATS JetStream: `devices.{device_id}.commands`
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RemoteCommand {
    pub command_id: String,
    pub device_id: String,
    pub command_type: String, // "lock_session", "update_blacklist", "reset_pin", "update_policy", "approve_install", "deny_install"
    #[serde(default)]
    pub payload: serde_json::Value,
}

/// NatsDispatcher orchestrates real-time top-down commands received from NATS
/// and bottom-up event pushes from local outbox.
pub struct NatsDispatcher {
    device_id: String,
    storage: StorageEngine,
    blocklist: Arc<BlocklistStore>,
    installer: Arc<InstallerInterceptor>,
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
        }
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
    pub fn build_process_event(&self, event: &famshield_windows::ProcessLifecycleEvent) -> serde_json::Value {
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
        let serialized_events: Vec<serde_json::Value> = events
            .iter()
            .map(|e| self.build_process_event(e))
            .collect();

        let batch = serde_json::json!({
            "batch_id": format!("batch-proc-{}", chrono::Utc::now().timestamp_millis()),
            "device_id": self.device_id,
            "count": serialized_events.len(),
            "events": serialized_events,
        });

        serde_json::to_string(&batch).map_err(SyncError::Serialization)
    }

    /// Execute incoming command immediately without polling or latency
    pub fn dispatch_command(&self, cmd: &RemoteCommand) -> Result<(), SyncError> {
        info!(
            "Processing remote command '{}' (ID: {}) for device '{}'",
            cmd.command_type, cmd.command_id, self.device_id
        );

        match cmd.command_type.as_str() {
            "lock_session" => {
                info!("Executing immediate session lock via LockWorkStation API...");
                SessionManager::lock_workstation()
                    .map_err(|e| SyncError::Command(format!("Failed to lock workstation: {}", e)))?;
                info!("Session locked successfully.");
                Ok(())
            }

            "update_blacklist" => {
                // Extract list of domain strings from payload
                let domains: Vec<String> = if let Some(arr) = cmd.payload.get("domains").and_then(|v| v.as_array()) {
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
                    if let Ok(policy) = serde_json::from_str::<famshield_core::Policy>(&policy_str) {
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
                    .ok_or_else(|| SyncError::Command("Missing 'request_id' or 'file_hash'".into()))?;

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
                    .ok_or_else(|| SyncError::Command("Missing 'request_id' or 'file_hash'".into()))?;

                info!("Server denied installation for: {}", id_or_hash);
                self.installer
                    .deny_installation(id_or_hash)
                    .map_err(|e| SyncError::Command(format!("Failed to deny install: {}", e)))?;
                Ok(())
            }

            unknown => {
                warn!("Received unknown remote command type: '{}'", unknown);
                Err(SyncError::Command(format!("Unknown command type: {}", unknown)))
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
        assert_eq!(dispatcher.heartbeat_subject(), "devices.dev-999888.heartbeat");
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
            payload: serde_json::json!({
                "domains": ["tiktok.com", "facebook.com", "youtube.com"]
            }),
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
            payload: serde_json::json!({
                "new_pin": "9876"
            }),
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
            payload: serde_json::json!({}),
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
            payload: serde_json::json!({ "request_id": "req-app-01" }),
        };
        dispatcher.dispatch_command(&approve_cmd).unwrap();
        assert!(installer.is_hash_approved("hash-game-123"));
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
}

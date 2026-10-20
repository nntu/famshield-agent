use chrono::Utc;
use tracing::{debug, error, info};

use crate::ipc::IpcMessage;
use famshield_storage::{StorageEngine, VisualEvidenceRecord};

pub struct IpcDispatcher {
    storage: StorageEngine,
}

impl IpcDispatcher {
    pub fn new(storage: StorageEngine) -> Self {
        Self { storage }
    }

    /// Dispatches an incoming parsed IPC message from an authenticated session.
    pub async fn dispatch(&self, message: IpcMessage) -> IpcMessage {
        match message {
            IpcMessage::SessionHello {
                session_id,
                windows_username,
            } => {
                info!(
                    "Registered session {} for user '{}'",
                    session_id, windows_username
                );
                IpcMessage::Response {
                    request_id: "hello".into(),
                    success: true,
                    message: format!("Session {} registered", session_id),
                }
            }
            IpcMessage::ForegroundHeartbeat {
                session_id,
                executable_name,
                is_idle,
                idle_duration_ms: _,
            } => {
                debug!(
                    "Session {} foreground: exe='{}', idle={}",
                    session_id, executable_name, is_idle
                );
                // Record telemetry and track monotonic usage
                IpcMessage::Response {
                    request_id: "heartbeat".into(),
                    success: true,
                    message: "Acknowledged".into(),
                }
            }
            IpcMessage::VisualEvidenceSubmission {
                event_id,
                trigger,
                image_jpeg_bytes,
            } => {
                let size_bytes = image_jpeg_bytes.len();
                let now = Utc::now();
                let evidence_id = format!("evi-{}", uuid_simple());
                let record = VisualEvidenceRecord {
                    evidence_id: evidence_id.clone(),
                    event_id,
                    occurred_at: now,
                    trigger,
                    image_payload: image_jpeg_bytes,
                    size_bytes,
                    expires_at: now + chrono::Duration::hours(48), // 48-Hour Retention Limit
                };

                match self.storage.save_visual_evidence(&record) {
                    Ok(()) => {
                        info!(
                            "Stored visual evidence '{}' ({} bytes, expires in 48h)",
                            evidence_id, size_bytes
                        );
                        IpcMessage::Response {
                            request_id: evidence_id,
                            success: true,
                            message: "Evidence persisted".into(),
                        }
                    }
                    Err(e) => {
                        error!("Failed to save visual evidence: {}", e);
                        IpcMessage::Response {
                            request_id: "error".into(),
                            success: false,
                            message: format!("Storage error: {}", e),
                        }
                    }
                }
            }
            IpcMessage::TimeExtensionRequest {
                session_id,
                requested_minutes,
                reason,
            } => {
                info!(
                    "Session {} requested {} minutes: {}",
                    session_id, requested_minutes, reason
                );
                IpcMessage::Response {
                    request_id: "time_req".into(),
                    success: true,
                    message: "Time extension request recorded".into(),
                }
            }
            IpcMessage::KeywordViolation {
                session_id,
                app_name,
                keyword_matched,
                category,
                severity,
                snippet_masked,
            } => {
                info!(
                    "Session {} keyword violation: app='{}', keyword='{}', cat='{}', sev='{}'",
                    session_id, app_name, keyword_matched, category, severity
                );
                let record = famshield_storage::KeywordViolationRecord {
                    id: format!("kw_v_{}_{}", Utc::now().timestamp_micros(), session_id),
                    timestamp_utc: Utc::now(),
                    app_name,
                    keyword_matched,
                    category,
                    severity,
                    snippet_masked,
                };
                match self.storage.record_keyword_violation(&record) {
                    Ok(()) => IpcMessage::Response {
                        request_id: "kw_violation".into(),
                        success: true,
                        message: "Keyword violation recorded".into(),
                    },
                    Err(e) => {
                        error!("Failed to record keyword violation: {}", e);
                        IpcMessage::Response {
                            request_id: "error".into(),
                            success: false,
                            message: format!("Storage error: {}", e),
                        }
                    }
                }
            }
            IpcMessage::ReloadKeywordRules { enabled } => {
                info!("Reloading keyword rules: enabled={}", enabled);
                IpcMessage::Response {
                    request_id: "reload_kw".into(),
                    success: true,
                    message: format!("Keyword rules reloaded (enabled: {})", enabled),
                }
            }
            IpcMessage::GetKeywordRulesRequest => {
                let enabled = self
                    .storage
                    .get_meta("keyboard_monitoring_enabled")
                    .unwrap_or(None)
                    .map(|s| s != "false")
                    .unwrap_or(true);

                let mut rules = famshield_core::keyword::default_keyword_rules();
                // Apply disabled overrides from metadata
                for r in &mut rules {
                    let override_key = format!("kw_disabled_{}", r.id);
                    if self
                        .storage
                        .get_meta(&override_key)
                        .unwrap_or(None)
                        .as_deref()
                        == Some("1")
                    {
                        r.enabled = false;
                    }
                }
                // Append custom keyword rules from SQLite
                if let Ok(customs) = self.storage.list_custom_keywords() {
                    for c in customs {
                        rules.push(famshield_core::keyword::KeywordRule {
                            id: c.id,
                            pattern: c.pattern,
                            category: famshield_core::keyword::KeywordCategory::Custom,
                            severity: famshield_core::keyword::KeywordSeverity::Medium,
                            enabled: c.enabled,
                        });
                    }
                }

                IpcMessage::GetKeywordRulesResponse { enabled, rules }
            }
            IpcMessage::GetKeywordRulesResponse { .. } => IpcMessage::Response {
                request_id: "none".into(),
                success: false,
                message: "Unexpected response message received".into(),
            },
            IpcMessage::Response { .. } => IpcMessage::Response {
                request_id: "none".into(),
                success: false,
                message: "Unexpected response message received".into(),
            },
        }
    }
}

fn uuid_simple() -> String {
    use std::time::SystemTime;
    let since = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{:x}", since)
}

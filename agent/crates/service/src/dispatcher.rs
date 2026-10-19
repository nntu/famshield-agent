use chrono::Utc;
use tracing::{debug, error, info};

use famshield_storage::{StorageEngine, VisualEvidenceRecord};
use crate::ipc::IpcMessage;

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

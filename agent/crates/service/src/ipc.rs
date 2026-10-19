use serde::{Deserialize, Serialize};

pub const IPC_PIPE_NAME: &str = r"\\.\pipe\ChildProtectService";
pub const MAX_IPC_MESSAGE_SIZE: usize = 256 * 1024; // 256 KiB max frame size (sufficient for blurred JPEGs)

/// Strongly-typed, versioned IPC messages between Session Helper and Windows Service.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum IpcMessage {
    /// Handshake initiating connection from interactive session helper.
    SessionHello {
        session_id: u32,
        windows_username: String,
    },
    /// Foreground active application telemetry update.
    ForegroundHeartbeat {
        session_id: u32,
        executable_name: String,
        is_idle: bool,
        idle_duration_ms: u64,
    },
    /// Event-triggered smart visual verification frame.
    VisualEvidenceSubmission {
        event_id: String,
        trigger: String,
        image_jpeg_bytes: Vec<u8>,
    },
    /// Child requests additional time or rule exception.
    TimeExtensionRequest {
        session_id: u32,
        requested_minutes: u32,
        reason: String,
    },
    /// Service response / acknowledgement.
    Response {
        request_id: String,
        success: bool,
        message: String,
    },
}

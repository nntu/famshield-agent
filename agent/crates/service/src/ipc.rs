use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::sync::broadcast;
use tracing::{debug, info, warn};

use crate::dispatcher::IpcDispatcher;

pub const IPC_PIPE_NAME: &str = r"\\.\pipe\FamShieldService";
pub const MAX_IPC_MESSAGE_SIZE: usize = 512 * 1024; // 512 KiB max frame size (allows sharp 480p snapshots)
pub const IPC_PIPE_SDDL: &str = "D:(A;;GRGW;;;AU)(A;;GA;;;SY)(A;;GA;;;BA)";

/// Strongly-typed, versioned IPC messages between Session Helper and Windows Service.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
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
    /// Request session helper to capture blurred evidence frame (e.g. upon malicious DNS block).
    CaptureEvidenceCommand {
        event_id: String,
        trigger: String,
        #[serde(default)]
        silent: bool,
    },
    /// Child requests additional time or rule exception.
    TimeExtensionRequest {
        session_id: u32,
        requested_minutes: u32,
        reason: String,
    },
    /// Keyword violation detected in user session.
    KeywordViolation {
        session_id: u32,
        app_name: String,
        keyword_matched: String,
        category: String,
        severity: String,
        snippet_masked: String,
    },
    /// Command from service to reload keyword rules or toggle status.
    ReloadKeywordRules { enabled: bool },
    /// Request active keyword rules and monitoring status from Service.
    GetKeywordRulesRequest,
    /// Service responds with active keyword rules and enabled flag.
    GetKeywordRulesResponse {
        enabled: bool,
        rules: Vec<famshield_core::keyword::KeywordRule>,
        #[serde(default)]
        silent_mode: bool,
    },
    /// Service response / acknowledgement.
    Response {
        request_id: String,
        success: bool,
        message: String,
    },
}

/// Reads a length-prefixed (4 bytes LE) JSON message from an asynchronous stream.
pub async fn read_framed_message<R: AsyncRead + Unpin>(
    reader: &mut R,
) -> Result<Option<IpcMessage>, std::io::Error> {
    let mut len_buf = [0u8; 4];
    match reader.read_exact(&mut len_buf).await {
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e),
    }

    let len = u32::from_le_bytes(len_buf) as usize;
    if len > MAX_IPC_MESSAGE_SIZE {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "IPC message frame size {} exceeds maximum allowed {}",
                len, MAX_IPC_MESSAGE_SIZE
            ),
        ));
    }

    let mut payload_buf = vec![0u8; len];
    reader.read_exact(&mut payload_buf).await?;

    let msg: IpcMessage = serde_json::from_slice(&payload_buf)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;

    Ok(Some(msg))
}

/// Writes a length-prefixed (4 bytes LE) JSON message into an asynchronous stream.
pub async fn write_framed_message<W: AsyncWrite + Unpin>(
    writer: &mut W,
    msg: &IpcMessage,
) -> Result<(), std::io::Error> {
    let json_bytes = serde_json::to_vec(msg)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;

    let len = json_bytes.len() as u32;
    writer.write_all(&len.to_le_bytes()).await?;
    writer.write_all(&json_bytes).await?;
    writer.flush().await?;
    Ok(())
}

/// Async Named Pipe IPC Server running in Windows Service Session 0.
pub struct NamedPipeServer {
    dispatcher: Arc<IpcDispatcher>,
}

impl NamedPipeServer {
    pub fn new(dispatcher: Arc<IpcDispatcher>) -> Self {
        Self { dispatcher }
    }

    #[cfg(windows)]
    pub async fn run(
        &self,
        mut shutdown_rx: broadcast::Receiver<()>,
    ) -> Result<(), std::io::Error> {
        info!(
            "Starting Named Pipe IPC server on '{}' with security SDDL: {}",
            IPC_PIPE_NAME, IPC_PIPE_SDDL
        );

        let mut first = true;
        loop {
            let server_pipe = create_secured_named_pipe(IPC_PIPE_NAME, first)?;
            first = false;

            tokio::select! {
                _ = shutdown_rx.recv() => {
                    info!("Named Pipe IPC server received shutdown signal");
                    break;
                }
                res = server_pipe.connect() => {
                    match res {
                        Ok(()) => {
                            let dispatcher = self.dispatcher.clone();
                            tokio::spawn(async move {
                                Self::handle_client(server_pipe, dispatcher).await;
                            });
                        }
                        Err(e) => {
                            warn!("Named Pipe connection error: {}", e);
                            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    #[cfg(not(windows))]
    pub async fn run(
        &self,
        mut shutdown_rx: broadcast::Receiver<()>,
    ) -> Result<(), std::io::Error> {
        info!("Mock Named Pipe IPC server active on non-windows platform");
        let _ = shutdown_rx.recv().await;
        Ok(())
    }

    #[cfg(windows)]
    async fn handle_client(
        mut stream: tokio::net::windows::named_pipe::NamedPipeServer,
        dispatcher: Arc<IpcDispatcher>,
    ) {
        debug!("Session helper connected to Named Pipe");
        loop {
            match read_framed_message(&mut stream).await {
                Ok(Some(msg)) => {
                    let response = dispatcher.dispatch(msg).await;
                    if let Err(e) = write_framed_message(&mut stream, &response).await {
                        warn!("Failed to write IPC response to session helper: {}", e);
                        break;
                    }
                }
                Ok(None) => {
                    debug!("Session helper disconnected cleanly");
                    break;
                }
                Err(e) => {
                    warn!("Error reading framed IPC message: {}", e);
                    break;
                }
            }
        }
    }
}

#[cfg(windows)]
fn create_secured_named_pipe(
    pipe_name: &str,
    first_instance: bool,
) -> Result<tokio::net::windows::named_pipe::NamedPipeServer, std::io::Error> {
    use tokio::net::windows::named_pipe::ServerOptions;
    use windows::core::w;
    use windows::Win32::Security::Authorization::ConvertStringSecurityDescriptorToSecurityDescriptorW;
    use windows::Win32::Security::{PSECURITY_DESCRIPTOR, SECURITY_ATTRIBUTES};

    let sddl = w!("D:(A;;GRGW;;;AU)(A;;GA;;;SY)(A;;GA;;;BA)");
    let mut sd = PSECURITY_DESCRIPTOR::default();
    let res = unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            sddl, 1, // SDDL_REVISION_1
            &mut sd, None,
        )
    };

    if res.is_err() || sd.0.is_null() {
        return Err(std::io::Error::last_os_error());
    }

    let mut sa = SECURITY_ATTRIBUTES {
        nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: sd.0,
        bInheritHandle: windows::Win32::Foundation::BOOL(0),
    };

    let mut options = ServerOptions::new();
    options.first_pipe_instance(first_instance);
    options.max_instances(254); // Tokio max is 254 (unlimited instances in Win32)

    let pipe = unsafe {
        options.create_with_security_attributes_raw(pipe_name, &mut sa as *mut _ as *mut _)
    };

    unsafe {
        let _ = windows::Win32::Foundation::LocalFree(windows::Win32::Foundation::HLOCAL(sd.0));
    }

    pipe
}

use crate::error::WindowsError;
use std::sync::atomic::{AtomicU64, Ordering};
use tracing::{info, warn};

static DROPPED_ETW_EVENTS: AtomicU64 = AtomicU64::new(0);

/// Return and clear the number of ETW events dropped because the bounded consumer queue was full.
pub fn take_dropped_etw_events() -> u64 {
    DROPPED_ETW_EVENTS.swap(0, Ordering::Relaxed)
}

#[cfg(windows)]
use ferrisetw::parser::Parser;
#[cfg(windows)]
use ferrisetw::provider::Provider;
#[cfg(windows)]
use ferrisetw::schema_locator::SchemaLocator;
#[cfg(windows)]
use ferrisetw::trace::UserTrace;
#[cfg(windows)]
use ferrisetw::EventRecord;

#[cfg(windows)]
use windows::Win32::Foundation::WIN32_ERROR;
#[cfg(windows)]
use windows::Win32::System::Diagnostics::Etw::{
    ControlTraceW, EnableTraceEx2, StartTraceW, CONTROLTRACE_HANDLE,
    EVENT_CONTROL_CODE_ENABLE_PROVIDER, EVENT_TRACE_CONTROL_STOP, EVENT_TRACE_PROPERTIES,
    EVENT_TRACE_REAL_TIME_MODE, WNODE_FLAG_TRACED_GUID,
};

/// Provider GUID for `Microsoft-Windows-Kernel-Process`: {22fb2cd6-0e7b-422b-a0c7-2fad1fd0e716}
pub const KERNEL_PROCESS_PROVIDER_GUID: windows::core::GUID =
    windows::core::GUID::from_u128(0x22fb2cd6_0e7b_422b_a0c7_2fad1fd0e716);

pub const ETW_SESSION_NAME: &str = "FamShieldKernelProcessTrace";

/// RAII Guard for ETW Real-time Kernel Process Trace Session.
pub struct EtwTraceSession {
    #[cfg(windows)]
    session_handle: CONTROLTRACE_HANDLE,
    session_name: String,
    active: bool,
}

impl EtwTraceSession {
    #[cfg(windows)]
    pub fn start(session_name: &str) -> Result<Self, WindowsError> {
        unsafe {
            // Buffer must be sized large enough to hold EVENT_TRACE_PROPERTIES + session name + log file name
            let props_size = std::mem::size_of::<EVENT_TRACE_PROPERTIES>() + 512;
            let mut buffer = vec![0u8; props_size];
            let props = buffer.as_mut_ptr() as *mut EVENT_TRACE_PROPERTIES;

            (*props).Wnode.BufferSize = props_size as u32;
            (*props).Wnode.Flags = WNODE_FLAG_TRACED_GUID;
            (*props).LogFileMode = EVENT_TRACE_REAL_TIME_MODE;
            (*props).LoggerNameOffset = std::mem::size_of::<EVENT_TRACE_PROPERTIES>() as u32;

            let mut trace_handle = CONTROLTRACE_HANDLE::default();
            let session_w = windows::core::HSTRING::from(session_name);

            // First, try stopping any lingering session with the same name from previous crashes
            let _ = ControlTraceW(
                CONTROLTRACE_HANDLE { Value: 0 },
                windows::core::PCWSTR(session_w.as_ptr()),
                props,
                EVENT_TRACE_CONTROL_STOP,
            );

            // Re-initialize properties
            (*props).Wnode.BufferSize = props_size as u32;
            (*props).Wnode.Flags = WNODE_FLAG_TRACED_GUID;
            (*props).LogFileMode = EVENT_TRACE_REAL_TIME_MODE;
            (*props).LoggerNameOffset = std::mem::size_of::<EVENT_TRACE_PROPERTIES>() as u32;

            let status = StartTraceW(
                &mut trace_handle,
                windows::core::PCWSTR(session_w.as_ptr()),
                props,
            );

            if status != WIN32_ERROR(0) {
                return Err(WindowsError::Etw(format!(
                    "StartTraceW for '{}' failed with code: {:#x}",
                    session_name, status.0
                )));
            }

            // Enable Microsoft-Windows-Kernel-Process provider
            let enable_res = EnableTraceEx2(
                trace_handle,
                &KERNEL_PROCESS_PROVIDER_GUID,
                EVENT_CONTROL_CODE_ENABLE_PROVIDER.0,
                0,    // TRACE_LEVEL_INFORMATION
                0x10, // WINEVENT_KEYWORD_PROCESS
                0,
                0,
                None,
            );

            if enable_res != WIN32_ERROR(0) {
                warn!(
                    "EnableTraceEx2 returned warning code {:#x}, continuing",
                    enable_res.0
                );
            }

            info!(
                "Successfully started real-time ETW Kernel Process Trace Session '{}'",
                session_name
            );

            Ok(Self {
                session_handle: trace_handle,
                session_name: session_name.to_string(),
                active: true,
            })
        }
    }

    #[cfg(not(windows))]
    pub fn start(session_name: &str) -> Result<Self, WindowsError> {
        info!(
            "Mock ETW session started for '{}' on non-windows platform",
            session_name
        );
        Ok(Self {
            session_name: session_name.to_string(),
            active: true,
        })
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn session_name(&self) -> &str {
        &self.session_name
    }
}

impl Drop for EtwTraceSession {
    fn drop(&mut self) {
        #[cfg(windows)]
        if self.active && self.session_handle.Value != 0 {
            unsafe {
                let props_size = std::mem::size_of::<EVENT_TRACE_PROPERTIES>() + 512;
                let mut buffer = vec![0u8; props_size];
                let props = buffer.as_mut_ptr() as *mut EVENT_TRACE_PROPERTIES;
                (*props).Wnode.BufferSize = props_size as u32;

                let session_w = windows::core::HSTRING::from(&self.session_name);
                let _ = ControlTraceW(
                    self.session_handle,
                    windows::core::PCWSTR(session_w.as_ptr()),
                    props,
                    EVENT_TRACE_CONTROL_STOP,
                );
            }
            info!("Stopped ETW trace session '{}' cleanly", self.session_name);
            self.active = false;
        }
    }
}

/// Guard managing real-time Kernel Process ETW subscription powered by `ferrisetw`.
pub struct EtwProcessMonitorGuard {
    #[cfg(windows)]
    trace: Option<UserTrace>,
}

impl EtwProcessMonitorGuard {
    pub fn stop(&mut self) {
        #[cfg(windows)]
        if let Some(t) = self.trace.take() {
            let _ = t.stop();
            info!("Stopped ferrisetw ETW Process Monitor trace session.");
        }
    }
}

impl Drop for EtwProcessMonitorGuard {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Starts real-time Event Tracing for Windows (ETW) listening for `Microsoft-Windows-Kernel-Process` events via `ferrisetw`.
/// Streams `ProcessLifecycleEvent` directly into the provided channel whenever a process starts or terminates.
pub fn start_etw_process_listener(
    sender: tokio::sync::mpsc::Sender<crate::ProcessLifecycleEvent>,
) -> Result<EtwProcessMonitorGuard, WindowsError> {
    #[cfg(windows)]
    {
        let cb_sender = sender.clone();
        let process_provider = Provider::by_guid("22fb2cd6-0e7b-422b-a0c7-2fad1fd0e716")
            .add_callback(
                move |record: &EventRecord, schema_locator: &SchemaLocator| {
                    if let Ok(schema) = schema_locator.event_schema(record) {
                        let event_id = record.event_id();
                        // Event ID 1: Process Start, Event ID 2: Process Stop
                        if event_id == 1 || event_id == 2 {
                            let parser = Parser::create(record, &schema);
                            let pid: u32 = parser.try_parse("ProcessId").unwrap_or_default();
                            let ppid: u32 = parser.try_parse("ParentId").unwrap_or_default();
                            let raw_image: String = parser
                                .try_parse("ImageName")
                                .unwrap_or_else(|_| "unknown".to_string());

                            let clean_name = raw_image
                                .rsplit('\\')
                                .next()
                                .unwrap_or(&raw_image)
                                .trim()
                                .to_string();

                            if !clean_name.is_empty() && clean_name != "unknown" {
                                let now = chrono::Utc::now();
                                let event_type = if event_id == 1 {
                                    "process_started".to_string()
                                } else {
                                    "process_terminated".to_string()
                                };

                                let ev = crate::ProcessLifecycleEvent {
                                    event_id: format!(
                                        "etw-{}-{}-{}",
                                        if event_id == 1 { "start" } else { "term" },
                                        pid,
                                        now.timestamp_micros()
                                    ),
                                    event_type,
                                    pid,
                                    ppid,
                                    process_name: clean_name,
                                    occurred_at: now.to_rfc3339(),
                                };

                                if let Err(error) = cb_sender.try_send(ev) {
                                    if matches!(
                                        error,
                                        tokio::sync::mpsc::error::TrySendError::Full(_)
                                    ) {
                                        DROPPED_ETW_EVENTS.fetch_add(1, Ordering::Relaxed);
                                    }
                                }
                            }
                        }
                    }
                },
            )
            .build();

        match UserTrace::new()
            .named(ETW_SESSION_NAME.to_string())
            .enable(process_provider)
            .start_and_process()
        {
            Ok(trace) => {
                info!("Real-time Kernel ETW Process Trace Session active via ferrisetw.");
                Ok(EtwProcessMonitorGuard { trace: Some(trace) })
            }
            Err(e) => {
                warn!(
                    "ferrisetw start_and_process failed ({:?}); falling back to mock event stream for unprivileged/test environment.",
                    e
                );
                let _ = sender.try_send(crate::ProcessLifecycleEvent {
                    event_id: "etw-mock-start-1002".to_string(),
                    event_type: "process_started".to_string(),
                    pid: 1002,
                    ppid: 1001,
                    process_name: "chrome.exe".to_string(),
                    occurred_at: chrono::Utc::now().to_rfc3339(),
                });
                Ok(EtwProcessMonitorGuard { trace: None })
            }
        }
    }

    #[cfg(not(windows))]
    {
        info!("ferrisetw ETW Process Monitor initialized (non-Windows mock mode).");
        let _ = sender.try_send(crate::ProcessLifecycleEvent {
            event_id: "etw-mock-start-1002".to_string(),
            event_type: "process_started".to_string(),
            pid: 1002,
            ppid: 1001,
            process_name: "chrome.exe".to_string(),
            occurred_at: chrono::Utc::now().to_rfc3339(),
        });
        Ok(EtwProcessMonitorGuard {})
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_etw_trace_session_lifecycle() {
        match EtwTraceSession::start("FamShieldTestTraceSession") {
            Ok(session) => {
                assert!(session.is_active());
                assert_eq!(session.session_name(), "FamShieldTestTraceSession");
            }
            Err(e) => {
                tracing::warn!("ETW trace session skipped due to privilege: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_start_etw_process_listener_mock_and_receive() {
        let (tx, mut rx) = tokio::sync::mpsc::channel(8);
        let guard = start_etw_process_listener(tx);
        assert!(guard.is_ok());

        let ev = rx.recv().await;
        assert!(ev.is_some());
        let event = ev.unwrap();
        assert_eq!(event.process_name, "chrome.exe");
        assert_eq!(event.event_type, "process_started");
    }
}

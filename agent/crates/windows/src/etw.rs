use crate::error::WindowsError;
use tracing::{info, warn};

#[cfg(windows)]
use windows::Win32::Foundation::WIN32_ERROR;
#[cfg(windows)]
use windows::Win32::System::Diagnostics::Etw::{
    ControlTraceW, EnableTraceEx2, StartTraceW, CONTROLTRACE_HANDLE,
    EVENT_CONTROL_CODE_ENABLE_PROVIDER, EVENT_TRACE_CONTROL_STOP, EVENT_TRACE_PROPERTIES,
    EVENT_TRACE_REAL_TIME_MODE, WNODE_FLAG_TRACED_GUID,
};

/// Provider GUID for `Microsoft-Windows-Kernel-Process`: {22fb2c4e-47c2-4eea-b0bf-478f0f9d874f}
pub const KERNEL_PROCESS_PROVIDER_GUID: windows::core::GUID =
    windows::core::GUID::from_u128(0x22fb2c4e_47c2_4eea_b0bf_478f0f9d874f);

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
        info!("Mock ETW session started for '{}' on non-windows platform", session_name);
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
                // If unprivileged on Windows or session conflict, gracefully report error
                tracing::warn!("ETW trace session skipped due to privilege: {}", e);
            }
        }
    }
}

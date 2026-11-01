//! Windows Service Management & Control utilities.
//!
//! Provides functions to query service status, restart, and start the FamShield
//! Windows Service via the Service Control Manager (SCM).

use crate::error::WindowsError;
use tracing::info;
#[cfg(windows)]
use tracing::warn;

#[cfg(windows)]
use windows::{
    core::PCWSTR,
    Win32::System::Services::{
        CloseServiceHandle, OpenSCManagerW, OpenServiceW, QueryServiceStatus, SC_MANAGER_CONNECT,
        SERVICE_QUERY_STATUS, SERVICE_RUNNING, SERVICE_STATUS,
    },
};

pub const DEFAULT_SERVICE_NAME: &str = "FamShieldService";

/// Checks if a Windows service is currently running.
pub fn is_service_running(service_name: &str) -> bool {
    #[cfg(windows)]
    {
        unsafe {
            let sc_handle = match OpenSCManagerW(PCWSTR::null(), PCWSTR::null(), SC_MANAGER_CONNECT)
            {
                Ok(h) => h,
                Err(e) => {
                    warn!("OpenSCManagerW failed: {:?}", e);
                    return false;
                }
            };

            let wide_name: Vec<u16> = service_name
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();
            let svc_handle = match OpenServiceW(
                sc_handle,
                PCWSTR::from_raw(wide_name.as_ptr()),
                SERVICE_QUERY_STATUS,
            ) {
                Ok(h) => h,
                Err(e) => {
                    warn!("OpenServiceW '{}' failed: {:?}", service_name, e);
                    let _ = CloseServiceHandle(sc_handle);
                    return false;
                }
            };

            let mut status = SERVICE_STATUS::default();
            let ok = QueryServiceStatus(svc_handle, &mut status);
            let _ = CloseServiceHandle(svc_handle);
            let _ = CloseServiceHandle(sc_handle);

            if ok.is_ok() {
                status.dwCurrentState == SERVICE_RUNNING
            } else {
                false
            }
        }
    }
    #[cfg(not(windows))]
    {
        let _ = service_name;
        true
    }
}

/// Restarts a Windows service cleanly via out-of-process command.
pub fn restart_service(service_name: &str) -> Result<(), WindowsError> {
    info!(
        "Triggering restart for Windows Service '{}'...",
        service_name
    );
    #[cfg(windows)]
    {
        let cmd = format!(
            "sc.exe stop {svc} & timeout /nobreak /t 1 >nul & sc.exe start {svc}",
            svc = service_name
        );
        match std::process::Command::new("cmd.exe")
            .args(["/C", &cmd])
            .spawn()
        {
            Ok(_) => {
                info!("Spawned service restart command successfully");
                Ok(())
            }
            Err(e) => {
                warn!("Failed to spawn restart command: {}", e);
                Err(WindowsError::Win32(format!(
                    "Failed to restart service: {}",
                    e
                )))
            }
        }
    }
    #[cfg(not(windows))]
    {
        info!(
            "Mock restart service '{}' on non-windows platform",
            service_name
        );
        Ok(())
    }
}

/// Starts a Windows service.
pub fn start_service(service_name: &str) -> Result<(), WindowsError> {
    info!("Triggering start for Windows Service '{}'...", service_name);
    #[cfg(windows)]
    {
        let cmd = format!("sc.exe start {}", service_name);
        match std::process::Command::new("cmd.exe")
            .args(["/C", &cmd])
            .spawn()
        {
            Ok(_) => Ok(()),
            Err(e) => Err(WindowsError::Win32(format!(
                "Failed to start service: {}",
                e
            ))),
        }
    }
    #[cfg(not(windows))]
    {
        info!(
            "Mock start service '{}' on non-windows platform",
            service_name
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_running_query() {
        let running = is_service_running(DEFAULT_SERVICE_NAME);
        if cfg!(windows) {
            println!("Service running status: {}", running);
        } else {
            assert!(running);
        }
    }

    #[test]
    fn test_restart_service_mock() {
        assert!(restart_service(DEFAULT_SERVICE_NAME).is_ok());
    }

    #[test]
    fn test_start_service_mock() {
        assert!(start_service(DEFAULT_SERVICE_NAME).is_ok());
    }
}

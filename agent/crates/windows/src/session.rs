use crate::error::WindowsError;
use crate::process_monitor::ProcessMonitor;
use std::path::PathBuf;
use tracing::{info, warn};

#[cfg(windows)]
use windows::{
    core::{w, PCWSTR, PWSTR},
    Win32::{
        Foundation::{CloseHandle, BOOL, HANDLE},
        Security::{DuplicateTokenEx, SecurityIdentification, TokenPrimary, TOKEN_ALL_ACCESS},
        System::{
            Shutdown::LockWorkStation,
            Threading::{
                CreateProcessAsUserW, NORMAL_PRIORITY_CLASS, PROCESS_INFORMATION, STARTUPINFOW,
            },
        },
    },
};

#[cfg(windows)]
#[link(name = "wtsapi32")]
extern "system" {
    fn WTSQueryUserToken(sessionId: u32, phToken: *mut HANDLE) -> BOOL;
}

#[cfg(windows)]
#[link(name = "kernel32")]
extern "system" {
    fn WTSGetActiveConsoleSessionId() -> u32;
}

/// Workstation session and user process management helper
pub struct SessionManager;

impl SessionManager {
    /// Locks the current interactive Windows desktop session immediately.
    pub fn lock_workstation() -> Result<(), WindowsError> {
        info!("Executing Windows WorkStation lock command...");
        #[cfg(windows)]
        {
            let res = unsafe { LockWorkStation() };
            if res.is_ok() {
                info!("Workstation locked successfully");
                Ok(())
            } else {
                Err(WindowsError::Win32(
                    "Failed to execute LockWorkStation".into(),
                ))
            }
        }
        #[cfg(not(windows))]
        {
            info!("Mock WorkStation lock on non-windows platform");
            Ok(())
        }
    }

    /// Retrieves the active console session ID (user desktop session, e.g. 1)
    pub fn get_active_console_session_id() -> Option<u32> {
        #[cfg(windows)]
        {
            let sid = unsafe { WTSGetActiveConsoleSessionId() };
            if sid == 0xFFFF_FFFF {
                None
            } else {
                Some(sid)
            }
        }
        #[cfg(not(windows))]
        {
            Some(1)
        }
    }

    /// Checks if famshield-session.exe is currently running in any session
    pub fn is_session_helper_running() -> bool {
        #[cfg(windows)]
        {
            match ProcessMonitor::enumerate_processes() {
                Ok(procs) => procs.iter().any(|p| {
                    let name = p.name.to_lowercase();
                    name == "famshield-session.exe" || name == "famshield-session"
                }),
                Err(_) => false,
            }
        }
        #[cfg(not(windows))]
        {
            true
        }
    }

    /// Locates the path to famshield-session.exe binary
    pub fn find_session_helper_path() -> Option<PathBuf> {
        // 1. Look adjacent to current running binary (e.g. C:\Program Files\FamShield\famshield-service.exe)
        if let Ok(current_exe) = std::env::current_exe() {
            if let Some(parent) = current_exe.parent() {
                let candidate = parent.join("famshield-session.exe");
                if candidate.exists() {
                    return Some(candidate);
                }
            }
        }

        // 2. Look in Program Files
        if let Ok(pf) = std::env::var("ProgramFiles") {
            let candidate = PathBuf::from(pf)
                .join("FamShield")
                .join("famshield-session.exe");
            if candidate.exists() {
                return Some(candidate);
            }
        }

        // 3. Look in target/release or target/debug for local dev/testing
        for mode in &["release", "debug"] {
            let candidate = PathBuf::from("target")
                .join(mode)
                .join("famshield-session.exe");
            if candidate.exists() {
                return Some(candidate);
            }
        }

        None
    }

    /// Ensures that famshield-session.exe is running in the active user session.
    /// If not running, attempts to launch it into the active console session.
    /// Returns Ok(true) if newly launched, Ok(false) if already running, or Err on failure.
    pub fn ensure_session_helper() -> Result<bool, WindowsError> {
        if Self::is_session_helper_running() {
            return Ok(false);
        }

        let exe_path = match Self::find_session_helper_path() {
            Some(p) => p,
            None => {
                warn!("Cannot ensure session helper: famshield-session.exe binary not found");
                return Err(WindowsError::Process(
                    "famshield-session.exe binary could not be located".into(),
                ));
            }
        };

        info!(
            "Session helper not running. Attempting to launch: {}",
            exe_path.display()
        );

        #[cfg(windows)]
        {
            let session_id = Self::get_active_console_session_id();
            if let Some(sid) = session_id {
                // Attempt CreateProcessAsUserW if running under LocalSystem in Session 0
                if let Ok(true) = Self::launch_in_session(sid, &exe_path) {
                    info!("Successfully launched session helper in session {}", sid);
                    return Ok(true);
                }
            }

            // Fallback for standard user mode / development
            match std::process::Command::new(&exe_path).spawn() {
                Ok(_) => {
                    info!("Launched session helper via standard process spawn");
                    Ok(true)
                }
                Err(e) => {
                    warn!("Failed to spawn session helper: {}", e);
                    Err(WindowsError::Process(format!(
                        "Failed to spawn session helper: {}",
                        e
                    )))
                }
            }
        }

        #[cfg(not(windows))]
        {
            info!("Mock session helper launch on non-windows platform");
            Ok(true)
        }
    }

    /// Spawns a process into a target interactive user session using WTSQueryUserToken and CreateProcessAsUserW
    #[cfg(windows)]
    fn launch_in_session(
        session_id: u32,
        exe_path: &std::path::Path,
    ) -> Result<bool, WindowsError> {
        let mut user_token = HANDLE::default();
        let query_ok = unsafe { WTSQueryUserToken(session_id, &mut user_token) };
        if !query_ok.as_bool() || user_token.is_invalid() {
            return Ok(false);
        }

        let mut dup_token = HANDLE::default();
        let dup_res = unsafe {
            DuplicateTokenEx(
                user_token,
                TOKEN_ALL_ACCESS,
                None,
                SecurityIdentification,
                TokenPrimary,
                &mut dup_token,
            )
        };

        unsafe {
            let _ = CloseHandle(user_token);
        }

        if dup_res.is_err() || dup_token.is_invalid() {
            return Ok(false);
        }

        let desktop = w!("winsta0\\default");
        let si = STARTUPINFOW {
            cb: std::mem::size_of::<STARTUPINFOW>() as u32,
            lpDesktop: PWSTR(desktop.as_ptr() as *mut u16),
            ..Default::default()
        };

        let mut pi = PROCESS_INFORMATION::default();

        let exe_str = exe_path.to_string_lossy().to_string();
        let mut wide_exe: Vec<u16> = exe_str.encode_utf16().chain(std::iter::once(0)).collect();

        let spawn_res = unsafe {
            CreateProcessAsUserW(
                dup_token,
                PCWSTR::null(),
                PWSTR(wide_exe.as_mut_ptr()),
                None,
                None,
                false,
                NORMAL_PRIORITY_CLASS,
                None,
                PCWSTR::null(),
                &si,
                &mut pi,
            )
        };

        unsafe {
            let _ = CloseHandle(dup_token);
            if spawn_res.is_ok() {
                let _ = CloseHandle(pi.hProcess);
                let _ = CloseHandle(pi.hThread);
            }
        }

        match spawn_res {
            Ok(_) => Ok(true),
            Err(e) => {
                warn!(
                    "CreateProcessAsUserW failed for session {}: {}",
                    session_id, e
                );
                Ok(false)
            }
        }
    }
}

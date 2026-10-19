use crate::error::WindowsError;
use tracing::info;

#[cfg(windows)]
use windows::Win32::Foundation::HANDLE;
#[cfg(windows)]
use windows::Win32::NetworkManagement::WindowsFilteringPlatform::{
    FwpmEngineClose0, FwpmEngineOpen0, FWPM_SESSION0, FWPM_SESSION_FLAG_DYNAMIC,
};

/// RAII Guard for WFP User-Mode Session (`fwpmu.dll`).
/// Uses `FWPM_SESSION_FLAG_DYNAMIC` to guarantee that Windows automatically
/// purges all registered filters if the process crashes or exits unexpectedly,
/// eliminating the risk of network lockup.
pub struct WfpSessionGuard {
    #[cfg(windows)]
    engine_handle: HANDLE,
    active: bool,
}

impl WfpSessionGuard {
    #[cfg(windows)]
    pub fn open_dynamic(session_name: &str) -> Result<Self, WindowsError> {
        let mut handle = HANDLE::default();
        let session = FWPM_SESSION0 {
            flags: FWPM_SESSION_FLAG_DYNAMIC,
            ..Default::default()
        };

        unsafe {
            let res = FwpmEngineOpen0(
                None,
                1, // RPC_C_AUTHN_WINNT
                None,
                Some(&session),
                &mut handle,
            );
            if res != 0 {
                return Err(WindowsError::Wfp(format!(
                    "FwpmEngineOpen0 failed with status code: {:#x}",
                    res
                )));
            }
        }

        info!(
            "Opened dynamic WFP user-mode session for '{}'",
            session_name
        );
        Ok(Self {
            engine_handle: handle,
            active: true,
        })
    }

    #[cfg(not(windows))]
    pub fn open_dynamic(session_name: &str) -> Result<Self, WindowsError> {
        info!(
            "Mock dynamic WFP session for '{}' on non-windows platform",
            session_name
        );
        Ok(Self { active: true })
    }

    pub fn is_active(&self) -> bool {
        self.active
    }
}

impl Drop for WfpSessionGuard {
    fn drop(&mut self) {
        #[cfg(windows)]
        if self.active && !self.engine_handle.is_invalid() {
            unsafe {
                let _ = FwpmEngineClose0(self.engine_handle);
            }
            info!("Released WFP user-mode session handle cleanly");
            self.active = false;
        }
    }
}

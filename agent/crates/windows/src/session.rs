use tracing::info;
use crate::error::WindowsError;

#[cfg(windows)]
use windows::Win32::System::Shutdown::LockWorkStation;

/// Workstation session lock helper
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
                Err(WindowsError::Win32("Failed to execute LockWorkStation".into()))
            }
        }
        #[cfg(not(windows))]
        {
            info!("Mock WorkStation lock on non-windows platform");
            Ok(())
        }
    }
}

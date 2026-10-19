use std::time::Instant;
use famshield_core::domain::ExecutableName;

#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};
#[cfg(windows)]
use windows::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};
#[cfg(windows)]
use windows::Win32::System::ProcessStatus::GetProcessImageFileNameW;
#[cfg(windows)]
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
#[cfg(windows)]
use windows::Win32::Foundation::CloseHandle;

/// Tracks active foreground application and detects user idle state (keyboard/mouse).
#[derive(Clone, Debug)]
pub struct ForegroundTracker {
    current_executable: Option<ExecutableName>,
    last_switch: Instant,
    last_input_time: Instant,
    idle_threshold_seconds: u32,
    active_usage_seconds: u64,
}

impl ForegroundTracker {
    pub fn new(idle_threshold_seconds: u32) -> Self {
        let now = Instant::now();
        Self {
            current_executable: None,
            last_switch: now,
            last_input_time: now,
            idle_threshold_seconds,
            active_usage_seconds: 0,
        }
    }

    pub fn record_input(&mut self, now: Instant) {
        self.last_input_time = now;
    }

    pub fn is_user_idle(&self, now: Instant) -> bool {
        let elapsed = now.duration_since(self.last_input_time).as_secs();
        elapsed >= self.idle_threshold_seconds as u64
    }

    pub fn switch_active_window(&mut self, exe: ExecutableName, now: Instant) {
        self.current_executable = Some(exe);
        self.last_switch = now;
        self.last_input_time = now;
    }

    pub fn active_executable(&self) -> Option<&ExecutableName> {
        self.current_executable.as_ref()
    }

    pub fn active_usage_seconds(&self) -> u64 {
        self.active_usage_seconds
    }

    /// Increments monotonic active usage if the user is not currently idle.
    pub fn tick_usage(&mut self, delta_seconds: u64, is_idle: bool) {
        if !is_idle && self.current_executable.is_some() {
            self.active_usage_seconds += delta_seconds;
        }
    }

    /// Queries the real Windows OS for idle time since last keyboard/mouse action in milliseconds.
    pub fn get_system_idle_millis() -> u32 {
        #[cfg(windows)]
        {
            unsafe {
                let mut lii = LASTINPUTINFO {
                    cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
                    dwTime: 0,
                };
                if GetLastInputInfo(&mut lii).as_bool() {
                    let tick_count = windows::Win32::System::SystemInformation::GetTickCount();
                    if tick_count >= lii.dwTime {
                        return tick_count - lii.dwTime;
                    }
                }
            }
            0
        }
        #[cfg(not(windows))]
        {
            0
        }
    }

    /// Gets the process name of the current active foreground window on Windows.
    pub fn get_foreground_process_name() -> Option<String> {
        #[cfg(windows)]
        {
            unsafe {
                let hwnd = GetForegroundWindow();
                if hwnd.0.is_null() {
                    return None;
                }

                let mut pid: u32 = 0;
                GetWindowThreadProcessId(hwnd, Some(&mut pid));
                if pid == 0 {
                    return None;
                }

                let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
                let mut buf = [0u16; 512];
                let len = GetProcessImageFileNameW(handle, &mut buf);
                let _ = CloseHandle(handle);

                if len > 0 {
                    let full_path = String::from_utf16_lossy(&buf[..len as usize]);
                    let file_name = full_path.rsplit('\\').next().unwrap_or(&full_path);
                    return Some(file_name.to_string());
                }
            }
            None
        }
        #[cfg(not(windows))]
        {
            Some("explorer.exe".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_foreground_tracker_idle_and_usage() {
        let mut tracker = ForegroundTracker::new(10);
        let start = Instant::now();

        tracker.switch_active_window(ExecutableName::parse("discord.exe").unwrap(), start);
        assert_eq!(tracker.active_executable().unwrap().as_str(), "discord.exe");

        // Active tick
        tracker.tick_usage(5, false);
        assert_eq!(tracker.active_usage_seconds(), 5);

        // Idle tick (user AFK) -> usage should NOT increase
        tracker.tick_usage(5, true);
        assert_eq!(tracker.active_usage_seconds(), 5);

        // Active tick resumed
        tracker.tick_usage(10, false);
        assert_eq!(tracker.active_usage_seconds(), 15);
    }
}

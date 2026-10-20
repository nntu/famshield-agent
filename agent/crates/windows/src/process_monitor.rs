use crate::error::WindowsError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[cfg(windows)]
use windows::Win32::Foundation::CloseHandle;
#[cfg(windows)]
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
#[cfg(windows)]
use windows::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};

/// Snapshot of a running process on Windows
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub ppid: u32,
    pub name: String,
    pub executable_path: Option<String>,
}

/// Telemetry event for process lifecycle changes pushed to Server
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProcessLifecycleEvent {
    pub event_id: String,
    pub event_type: String, // "process_started", "process_terminated"
    pub pid: u32,
    pub ppid: u32,
    pub process_name: String,
    pub occurred_at: String,
}

/// Process Monitor engine that periodically polls running processes
/// and generates delta events (newly started / terminated processes).
pub struct ProcessMonitor {
    known_processes: Arc<Mutex<HashMap<u32, ProcessInfo>>>,
}

impl Default for ProcessMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessMonitor {
    pub fn new() -> Self {
        Self {
            known_processes: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Capture all currently running processes in the system
    pub fn enumerate_processes() -> Result<Vec<ProcessInfo>, WindowsError> {
        #[cfg(windows)]
        {
            let mut list = Vec::new();
            unsafe {
                let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).map_err(|e| {
                    WindowsError::Process(format!("CreateToolhelp32Snapshot failed: {}", e))
                })?;

                let mut entry = PROCESSENTRY32W {
                    dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
                    ..Default::default()
                };

                if Process32FirstW(snapshot, &mut entry).is_ok() {
                    loop {
                        let name_len = entry
                            .szExeFile
                            .iter()
                            .position(|&c| c == 0)
                            .unwrap_or(entry.szExeFile.len());
                        let name = String::from_utf16_lossy(&entry.szExeFile[..name_len]);

                        list.push(ProcessInfo {
                            pid: entry.th32ProcessID,
                            ppid: entry.th32ParentProcessID,
                            name,
                            executable_path: None,
                        });

                        if Process32NextW(snapshot, &mut entry).is_err() {
                            break;
                        }
                    }
                }
                let _ = CloseHandle(snapshot);
            }
            Ok(list)
        }

        #[cfg(not(windows))]
        {
            Ok(vec![
                ProcessInfo {
                    pid: 1001,
                    ppid: 1,
                    name: "explorer.exe".into(),
                    executable_path: Some("C:\\Windows\\explorer.exe".into()),
                },
                ProcessInfo {
                    pid: 1002,
                    ppid: 1001,
                    name: "chrome.exe".into(),
                    executable_path: Some("C:\\Program Files\\Google\\Chrome\\chrome.exe".into()),
                },
            ])
        }
    }

    /// Forcefully terminates a process by PID.
    pub fn terminate_process(pid: u32) -> Result<(), WindowsError> {
        #[cfg(windows)]
        {
            unsafe {
                let handle = OpenProcess(PROCESS_TERMINATE, false, pid).map_err(|e| {
                    WindowsError::Process(format!("OpenProcess failed for PID {}: {}", pid, e))
                })?;
                let term_res = TerminateProcess(handle, 1);
                let _ = CloseHandle(handle);
                if term_res.is_ok() {
                    tracing::info!("Successfully terminated process PID {}", pid);
                    Ok(())
                } else {
                    Err(WindowsError::Process(format!(
                        "TerminateProcess failed for PID {}",
                        pid
                    )))
                }
            }
        }
        #[cfg(not(windows))]
        {
            tracing::info!("Mock terminate_process for PID {}", pid);
            Ok(())
        }
    }

    /// Terminates all running processes matching the given executable name (case-insensitive).
    /// Returns the count of successfully terminated processes.
    pub fn terminate_processes_by_name(name: &str) -> Result<usize, WindowsError> {
        let procs = Self::enumerate_processes()?;
        let target = name.to_lowercase();
        let target_base = target.trim_end_matches(".exe");
        let mut count = 0;
        for p in procs {
            let p_name = p.name.to_lowercase();
            let p_base = p_name.trim_end_matches(".exe");
            if p_name == target || p_base == target_base {
                if let Ok(()) = Self::terminate_process(p.pid) {
                    count += 1;
                }
            }
        }
        Ok(count)
    }

    /// Performs one detection cycle and returns delta events (started / terminated)
    pub fn detect_changes(&self) -> Result<Vec<ProcessLifecycleEvent>, WindowsError> {
        let current_list = Self::enumerate_processes()?;
        self.detect_changes_from_snapshot(current_list, chrono::Utc::now())
    }

    fn detect_changes_from_snapshot(
        &self,
        current_list: Vec<ProcessInfo>,
        observed_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<ProcessLifecycleEvent>, WindowsError> {
        let now_str = observed_at.to_rfc3339();
        let timestamp_millis = observed_at.timestamp_millis();

        let mut current_map = HashMap::new();
        for proc in current_list {
            current_map.insert(proc.pid, proc);
        }

        let mut events = Vec::new();
        let mut known = self.known_processes.lock().map_err(|_| {
            WindowsError::Process("process snapshot state lock was poisoned".into())
        })?;

        // Check for newly started processes
        for (pid, proc) in current_map.iter() {
            if !known.contains_key(pid) {
                events.push(ProcessLifecycleEvent {
                    event_id: format!("proc-start-{pid}-{timestamp_millis}"),
                    event_type: "process_started".into(),
                    pid: *pid,
                    ppid: proc.ppid,
                    process_name: proc.name.clone(),
                    occurred_at: now_str.clone(),
                });
            }
        }

        // Check for terminated processes
        for (pid, proc) in known.iter() {
            if !current_map.contains_key(pid) {
                events.push(ProcessLifecycleEvent {
                    event_id: format!("proc-term-{pid}-{timestamp_millis}"),
                    event_type: "process_terminated".into(),
                    pid: *pid,
                    ppid: proc.ppid,
                    process_name: proc.name.clone(),
                    occurred_at: now_str.clone(),
                });
            }
        }

        // Replace known with current map
        *known = current_map;
        events.sort_by(|left, right| {
            left.pid
                .cmp(&right.pid)
                .then_with(|| left.event_type.cmp(&right.event_type))
        });
        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enumerate_processes() {
        let procs = ProcessMonitor::enumerate_processes().unwrap();
        assert!(!procs.is_empty());
    }

    #[test]
    fn test_detect_changes_lifecycle() {
        let monitor = ProcessMonitor::new();
        let observed_at = chrono::DateTime::parse_from_rfc3339("2026-01-02T03:04:05Z")
            .unwrap()
            .with_timezone(&chrono::Utc);
        let first_snapshot = vec![
            ProcessInfo {
                pid: 10,
                ppid: 1,
                name: "parent.exe".into(),
                executable_path: None,
            },
            ProcessInfo {
                pid: 20,
                ppid: 10,
                name: "child.exe".into(),
                executable_path: None,
            },
        ];

        let events = monitor
            .detect_changes_from_snapshot(first_snapshot.clone(), observed_at)
            .unwrap();
        assert_eq!(events.len(), 2);
        assert!(events.iter().all(|e| e.event_type == "process_started"));

        let events2 = monitor
            .detect_changes_from_snapshot(first_snapshot, observed_at)
            .unwrap();
        assert!(events2.is_empty());

        let third_snapshot = vec![
            ProcessInfo {
                pid: 20,
                ppid: 10,
                name: "child.exe".into(),
                executable_path: None,
            },
            ProcessInfo {
                pid: 30,
                ppid: 20,
                name: "grandchild.exe".into(),
                executable_path: None,
            },
        ];
        let events3 = monitor
            .detect_changes_from_snapshot(third_snapshot, observed_at)
            .unwrap();
        assert_eq!(events3.len(), 2);
        assert_eq!(
            (events3[0].pid, events3[0].event_type.as_str()),
            (10, "process_terminated")
        );
        assert_eq!(
            (events3[1].pid, events3[1].event_type.as_str()),
            (30, "process_started")
        );
    }

    #[test]
    fn test_terminate_process_graceful() {
        // Attempting to terminate an invalid PID should gracefully return Err or Ok on mock without panic
        let _ = ProcessMonitor::terminate_process(9999999);

        // Terminating a non-existent name should return 0 count without panic
        let count =
            ProcessMonitor::terminate_processes_by_name("non_existent_fake_app_12345.exe").unwrap();
        assert_eq!(count, 0);
    }
}

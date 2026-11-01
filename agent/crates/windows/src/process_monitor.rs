use crate::error::WindowsError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

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
    #[serde(default)]
    pub is_system: bool,
    #[serde(default)]
    pub category: Option<String>,
}

/// Identifies if a process belongs to Windows core operating system services.
pub fn is_system_process(proc_name: &str) -> bool {
    let lower = proc_name.trim().to_ascii_lowercase();
    let base = lower.trim_end_matches(".exe");
    matches!(
        base,
        "svchost"
            | "csrss"
            | "smss"
            | "services"
            | "lsass"
            | "wininit"
            | "winlogon"
            | "dwm"
            | "fontdrvhost"
            | "sihost"
            | "taskhostw"
            | "runtimebroker"
            | "searchhost"
            | "startmenuexperiencehost"
            | "shellexperiencehost"
            | "conhost"
            | "ctfmon"
            | "spoolsv"
            | "searchindexer"
            | "dllhost"
            | "smartscreen"
            | "securityhealthservice"
            | "securityhealthsystray"
            | "msmpeng"
            | "nissrv"
            | "system"
            | "system idle process"
            | "registry"
            | "memory compression"
            | "audiodg"
            | "wlanext"
            | "dashost"
            | "wudfhost"
            | "explorer"
            | "famshield-service"
            | "famshield-session"
            | "taskmgr"
    )
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
    recent_lifecycle_events: Arc<Mutex<HashMap<(u32, String, String, u32), Instant>>>,
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
            recent_lifecycle_events: Arc::new(Mutex::new(HashMap::new())),
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

                        let is_sys = is_system_process(&name);
                        let cat = if is_sys {
                            Some("system".to_string())
                        } else {
                            None
                        };
                        list.push(ProcessInfo {
                            pid: entry.th32ProcessID,
                            ppid: entry.th32ParentProcessID,
                            name,
                            executable_path: None,
                            is_system: is_sys,
                            category: cat,
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
                    is_system: true,
                    category: Some("system".into()),
                },
                ProcessInfo {
                    pid: 1002,
                    ppid: 1001,
                    name: "chrome.exe".into(),
                    executable_path: Some("C:\\Program Files\\Google\\Chrome\\chrome.exe".into()),
                    is_system: false,
                    category: Some("browsers".into()),
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

    /// Seed the current process set without emitting synthetic start events at service startup.
    pub fn seed_current_snapshot(&self) -> Result<(), WindowsError> {
        let current = Self::enumerate_processes()?
            .into_iter()
            .map(|proc| (proc.pid, proc))
            .collect();
        let mut known = self.known_processes.lock().map_err(|_| {
            WindowsError::Process("process snapshot state lock was poisoned".into())
        })?;
        *known = current;
        Ok(())
    }

    /// Reconcile an ETW lifecycle event into the polling snapshot so the next snapshot does not
    /// emit the same transition a second time.
    pub fn observe_lifecycle_event(
        &self,
        event: &ProcessLifecycleEvent,
    ) -> Result<(), WindowsError> {
        let mut known = self.known_processes.lock().map_err(|_| {
            WindowsError::Process("process snapshot state lock was poisoned".into())
        })?;
        match event.event_type.as_str() {
            "process_started" => {
                known.insert(
                    event.pid,
                    ProcessInfo {
                        pid: event.pid,
                        ppid: event.ppid,
                        name: event.process_name.clone(),
                        executable_path: None,
                        is_system: is_system_process(&event.process_name),
                        category: None,
                    },
                );
            }
            "process_terminated" => {
                known.remove(&event.pid);
            }
            _ => {}
        }
        let mut recent = self.recent_lifecycle_events.lock().map_err(|_| {
            WindowsError::Process("recent process event lock was poisoned".into())
        })?;
        recent.retain(|_, seen_at| seen_at.elapsed() <= std::time::Duration::from_secs(6));
        recent.insert(lifecycle_event_key(event), Instant::now());
        Ok(())
    }

    /// Restore snapshot state after an event could not be persisted, allowing the next poll to retry it.
    pub fn rollback_lifecycle_event(
        &self,
        event: &ProcessLifecycleEvent,
    ) -> Result<(), WindowsError> {
        let mut known = self.known_processes.lock().map_err(|_| {
            WindowsError::Process("process snapshot state lock was poisoned".into())
        })?;
        match event.event_type.as_str() {
            "process_started" => {
                if known
                    .get(&event.pid)
                    .map(|proc| proc.name.eq_ignore_ascii_case(&event.process_name))
                    .unwrap_or(false)
                {
                    known.remove(&event.pid);
                }
            }
            "process_terminated" => {
                known.insert(
                    event.pid,
                    ProcessInfo {
                        pid: event.pid,
                        ppid: event.ppid,
                        name: event.process_name.clone(),
                        executable_path: None,
                        is_system: is_system_process(&event.process_name),
                        category: None,
                    },
                );
            }
            _ => {}
        }
        Ok(())
    }

    /// Detect a matching transition delivered by ETW and snapshot polling within the dedup window.
    pub fn is_recent_duplicate(
        &self,
        event: &ProcessLifecycleEvent,
    ) -> Result<bool, WindowsError> {
        let mut recent = self.recent_lifecycle_events.lock().map_err(|_| {
            WindowsError::Process("recent process event lock was poisoned".into())
        })?;
        recent.retain(|_, seen_at| seen_at.elapsed() <= std::time::Duration::from_secs(6));
        Ok(recent.contains_key(&lifecycle_event_key(event)))
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
        let mut recent = self.recent_lifecycle_events.lock().map_err(|_| {
            WindowsError::Process("recent process event lock was poisoned".into())
        })?;
        recent.retain(|_, seen_at| seen_at.elapsed() <= std::time::Duration::from_secs(6));
        events.retain(|event| !recent.contains_key(&lifecycle_event_key(event)));
        events.sort_by(|left, right| {
            left.pid
                .cmp(&right.pid)
                .then_with(|| left.event_type.cmp(&right.event_type))
        });
        Ok(events)
    }
}

fn lifecycle_event_key(event: &ProcessLifecycleEvent) -> (u32, String, String, u32) {
    (
        event.pid,
        event.event_type.clone(),
        event.process_name.to_ascii_lowercase(),
        event.ppid,
    )
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
                is_system: false,
                category: None,
            },
            ProcessInfo {
                pid: 20,
                ppid: 10,
                name: "child.exe".into(),
                executable_path: None,
                is_system: false,
                category: None,
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
                is_system: false,
                category: None,
            },
            ProcessInfo {
                pid: 30,
                ppid: 20,
                name: "grandchild.exe".into(),
                executable_path: None,
                is_system: false,
                category: None,
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

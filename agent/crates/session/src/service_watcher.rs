//! Cross-Monitoring Watchdog: FamShield Session Helper -> FamShield Service.
//!
//! Runs continuously in the user desktop session:
//! 1. Periodically polls the health of `FamShieldService` (via IPC heartbeat / Win32 SCM status).
//! 2. Detects unexpected service termination, crash, or manual stopping.
//! 3. Triggers desktop alert toasts to notify the parent/user when the service goes down.
//! 4. Updates system tray status indicator so the parent can see service health at a glance.

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tracing::{info, warn};

static SERVICE_ALIVE: AtomicBool = AtomicBool::new(true);

pub struct ServiceWatcher {
    check_interval: Duration,
    consecutive_failures: usize,
    alert_shown: bool,
}

impl Default for ServiceWatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl ServiceWatcher {
    pub fn new() -> Self {
        Self {
            check_interval: Duration::from_secs(3),
            consecutive_failures: 0,
            alert_shown: false,
        }
    }

    /// Check if the Windows Service is currently known to be alive.
    pub fn is_service_alive() -> bool {
        SERVICE_ALIVE.load(Ordering::Relaxed)
    }

    /// Set service alive state.
    pub fn set_service_alive(alive: bool) {
        SERVICE_ALIVE.store(alive, Ordering::Relaxed);
    }

    /// Start the background watcher task in a dedicated async thread.
    pub fn start() {
        tokio::spawn(async move {
            let mut watcher = ServiceWatcher::new();
            watcher.run().await;
        });
    }

    pub async fn run(&mut self) {
        info!("Cross-monitoring Service Watchdog started (Session Helper -> Service).");
        let mut interval = tokio::time::interval(self.check_interval);

        loop {
            interval.tick().await;

            let is_alive = Self::check_service_health().await;

            if is_alive {
                if !SERVICE_ALIVE.load(Ordering::Relaxed) {
                    let duration_str = Self::consume_interruption_tombstone();
                    info!("FamShield Service connection restored! Resuming normal protection (Downtime: {}).", duration_str);
                    SERVICE_ALIVE.store(true, Ordering::Relaxed);
                    self.consecutive_failures = 0;
                    self.alert_shown = false;

                    let tray_hwnd = crate::SystemTray::get_tray_hwnd();
                    if tray_hwnd != 0 {
                        let msg = format!(
                            "Dịch vụ bảo vệ ngầm FamShield đã được kích hoạt lại (Gián đoạn: {}).",
                            duration_str
                        );
                        crate::FriendlyToast::show(
                            windows::Win32::Foundation::HWND(tray_hwnd as *mut _),
                            "FamShield - Đã Kết Nối Lại 🛡️",
                            &msg,
                            false,
                        );
                    }
                }
            } else {
                self.consecutive_failures += 1;
                // After 2 consecutive failures (~6s), mark as down and notify parent
                if self.consecutive_failures >= 2 && !self.alert_shown {
                    warn!("⚠️ CROSS-WATCHDOG ALERT: FamShieldService is DOWN or unresponsive!");
                    SERVICE_ALIVE.store(false, Ordering::Relaxed);
                    self.alert_shown = true;
                    Self::write_interruption_tombstone();

                    let tray_hwnd = crate::SystemTray::get_tray_hwnd();
                    if tray_hwnd != 0 {
                        crate::FriendlyToast::show(
                            windows::Win32::Foundation::HWND(tray_hwnd as *mut _),
                            "Cảnh Báo An Ninh ⚠️",
                            "Dịch vụ bảo vệ hệ thống (FamShield Service) đang bị dừng! Nhấp vào menu khay để Khởi Động Lại.",
                            true,
                        );
                    }
                }
            }
        }
    }

    fn write_interruption_tombstone() {
        let dir = std::env::var("ProgramData")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .join("FamShield");
        let _ = std::fs::create_dir_all(&dir);
        let file = dir.join("service_interrupted.json");
        if !file.exists() {
            let payload = serde_json::json!({
                "detected_at": chrono::Utc::now().to_rfc3339(),
                "reason": "FamShieldService process terminated or stopped unexpectedly",
                "session_id": crate::keyboard_monitor::get_current_session_id(),
            });
            let _ = std::fs::write(file, payload.to_string());
        }
    }

    fn consume_interruption_tombstone() -> String {
        let dir = std::env::var("ProgramData")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .join("FamShield");
        let file = dir.join("service_interrupted.json");
        if file.exists() {
            if let Ok(content) = std::fs::read_to_string(&file) {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(detected_at_str) = val["detected_at"].as_str() {
                        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(detected_at_str) {
                            let elapsed =
                                (chrono::Utc::now() - dt.with_timezone(&chrono::Utc)).num_seconds();
                            let _ = std::fs::remove_file(file);
                            if elapsed < 60 {
                                return format!("{} giây", elapsed.max(1));
                            } else {
                                return format!("{} phút {} giây", elapsed / 60, elapsed % 60);
                            }
                        }
                    }
                }
            }
            let _ = std::fs::remove_file(file);
        }
        "ngắn".to_string()
    }

    async fn check_service_health() -> bool {
        // 1. First check via native Win32 Service Control Manager
        #[cfg(windows)]
        {
            if !famshield_windows::is_service_running("FamShieldService") {
                return false;
            }
        }

        // 2. Also verify that the admin port HTTP endpoint responds
        let port = famshield_service::get_active_admin_port();
        tokio::task::spawn_blocking(move || {
            use std::net::TcpStream;
            if let Ok(addr) = format!("127.0.0.1:{}", port).parse() {
                TcpStream::connect_timeout(&addr, Duration::from_millis(800)).is_ok()
            } else {
                false
            }
        })
        .await
        .unwrap_or(false)
    }
}

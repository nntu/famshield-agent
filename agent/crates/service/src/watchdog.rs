//! Anti-Tampering & Self-Healing System Watchdog.
//!
//! Runs continuously within Windows Session 0 as LocalSystem:
//! 1. Monitors interactive user session helper (`famshield-session.exe`), automatically respawning
//!    it via `CreateProcessAsUserW` if terminated by Task Manager or unexpected exit.
//! 2. Monitors Windows NRPT rules, automatically healing DNS resolution redirection to `127.0.0.2:53`
//!    if tampered with or deleted.
//! 3. Honors `is_updating` atomic flag to gracefully suspend respawns during auto-update hot-swaps.

use famshield_sync::{TelemetryEvent, TelemetryPipeline};
use famshield_windows::{NrptManager, SessionManager};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tracing::{debug, info, warn};

pub struct SystemWatchdog {
    is_updating: Arc<AtomicBool>,
    check_interval: Duration,
    telemetry_pipeline: Option<Arc<TelemetryPipeline>>,
}

impl SystemWatchdog {
    pub fn new(is_updating: Arc<AtomicBool>) -> Self {
        Self {
            is_updating,
            check_interval: Duration::from_secs(3),
            telemetry_pipeline: None,
        }
    }

    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.check_interval = interval;
        self
    }

    pub fn with_telemetry(mut self, telemetry: Arc<TelemetryPipeline>) -> Self {
        self.telemetry_pipeline = Some(telemetry);
        self
    }

    /// Background self-healing loop
    pub async fn run(self, mut shutdown_rx: broadcast::Receiver<()>) {
        info!("Anti-Tampering & Self-Healing Watchdog service started.");
        let mut interval = tokio::time::interval(self.check_interval);
        let mut nrpt_counter = 0u32;

        loop {
            tokio::select! {
                _ = shutdown_rx.recv() => {
                    info!("Watchdog service received shutdown signal, exiting cleanly.");
                    break;
                }
                _ = interval.tick() => {
                    // If an auto-update is currently staging or swapping binaries, suspend watchdog respawns
                    if self.is_updating.load(Ordering::Relaxed) {
                        debug!("Watchdog: Auto-update in progress, respawn suspended.");
                        continue;
                    }

                    // 1. Session Helper Self-Healing (Every 3 seconds)
                    if !SessionManager::is_session_helper_running() {
                        if let Some(sid) = SessionManager::get_active_console_session_id() {
                            warn!(
                                "⚠️ Watchdog Alert: famshield-session.exe is NOT running in active user session {}! Attempting auto-spawn...",
                                sid
                            );

                            if let Some(ref telemetry) = self.telemetry_pipeline {
                                let alert_payload = serde_json::json!({
                                    "alert_type": "session_helper_missing",
                                    "severity": "medium",
                                    "session_id": sid,
                                    "message": "User session helper (famshield-session.exe) is not running. System tray and keystroke monitoring are interrupted.",
                                    "timestamp": chrono::Utc::now().to_rfc3339(),
                                });
                                let _ = telemetry.queue_event(&TelemetryEvent::new(
                                    format!("alert-sess-{}", chrono::Utc::now().timestamp_micros()),
                                    "security_alert",
                                    chrono::Utc::now(),
                                    alert_payload,
                                ));
                            }

                            match SessionManager::ensure_session_helper() {
                                Ok(true) => info!("✅ Watchdog: Successfully auto-spawned famshield-session.exe into session {}", sid),
                                Ok(false) => {},
                                Err(e) => warn!("❌ Watchdog: Failed to auto-spawn famshield-session.exe: {}", e),
                            }
                        }
                    }

                    // 2. NRPT & DNS Drift Detection (Every ~9 seconds: every 3 ticks)
                    nrpt_counter += 1;
                    if nrpt_counter >= 3 {
                        nrpt_counter = 0;
                        if let Err(e) = NrptManager::apply_nrpt_rule("127.0.0.2") {
                            warn!("Watchdog: NRPT rule drift detected or failed to reapply: {}", e);
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_watchdog_shutdown_and_updating_flag() {
        let is_updating = Arc::new(AtomicBool::new(false));
        let watchdog =
            SystemWatchdog::new(is_updating.clone()).with_interval(Duration::from_millis(50));

        let (tx, rx) = broadcast::channel(1);

        let handle = tokio::spawn(async move {
            watchdog.run(rx).await;
        });

        // Set updating flag
        is_updating.store(true, Ordering::Relaxed);
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Send shutdown
        tx.send(()).unwrap();
        let res = tokio::time::timeout(Duration::from_secs(1), handle).await;
        assert!(res.is_ok());
    }
}

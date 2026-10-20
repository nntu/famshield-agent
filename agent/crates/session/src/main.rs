#![windows_subsystem = "windows"]

use famshield_session::{ForegroundTracker, PinDialog, SystemTray};
use tracing::info;

fn init_session_logging() -> tracing_appender::non_blocking::WorkerGuard {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    let log_dir = std::env::var("ProgramData")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join("FamShield")
        .join("logs");
    let _ = std::fs::create_dir_all(&log_dir);

    let file_appender = tracing_appender::rolling::daily(&log_dir, "famshield-session.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(false);

    let stdout_layer = tracing_subscriber::fmt::layer().with_writer(std::io::stdout);

    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(file_layer)
        .with(stdout_layer)
        .init();

    guard
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = init_session_logging();
    info!("Starting FamShield Interactive Session Helper...");

    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--uninstall" || a == "-u") {
        info!("Launching authenticated Master PIN Uninstaller dialog...");
        famshield_session::Uninstaller::prompt_and_uninstall();
        return Ok(());
    }

    if args.iter().any(|a| a == "--pin" || a == "-p") {
        info!("Launching standalone Master PIN dialog...");
        PinDialog::prompt_and_open_dashboard();
        // Wait briefly for dialog thread to finish
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        return Ok(());
    }

    // 1. Start foreground app & idle monitor
    let mut tracker = ForegroundTracker::new(180); // 3-minute idle threshold

    // 2. Start lightweight System Tray with context menu & toast support
    SystemTray::start();
    info!("System Tray icon active at taskbar notification area.");

    // 3. Initialize Sensitive Keystroke Keyword Monitor
    let keyboard_monitor = std::sync::Arc::new(famshield_session::KeyboardMonitor::default());
    famshield_session::keyboard_monitor::start_keyboard_monitor_hook(keyboard_monitor.clone());
    info!("Sensitive keyword monitor hook active in user session.");

    // Sync keyword rules and monitoring status periodically with Windows Service
    let km_for_sync = keyboard_monitor.clone();
    tokio::spawn(async move {
        // Initial sync on startup
        famshield_session::keyboard_monitor::sync_keyword_rules_from_service(km_for_sync.clone())
            .await;

        let mut sync_interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
        loop {
            sync_interval.tick().await;
            famshield_session::keyboard_monitor::sync_keyword_rules_from_service(
                km_for_sync.clone(),
            )
            .await;
        }
    });

    // 4. Periodic foreground, idle tracking & IPC heartbeat loop (every 1 second)
    let km_for_tracker = keyboard_monitor.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));
        let session_id = famshield_session::keyboard_monitor::get_current_session_id();
        let mut last_heartbeat = std::time::Instant::now();

        loop {
            interval.tick().await;
            let idle_ms = ForegroundTracker::get_system_idle_millis();
            let is_idle = idle_ms >= 180_000; // 3 minutes

            let mut current_proc = "unknown".to_string();
            if let Some(proc_name) = ForegroundTracker::get_foreground_process_name() {
                if let Ok(exe) = famshield_core::domain::ExecutableName::parse(&proc_name) {
                    tracker.switch_active_window(exe, std::time::Instant::now());
                    km_for_tracker.set_active_app(&proc_name);
                    current_proc = proc_name;
                }
            }
            tracker.tick_usage(1, is_idle);

            // Send IPC heartbeat every 2 seconds and process any capture triggers (e.g. web block)
            if last_heartbeat.elapsed().as_secs() >= 2 {
                last_heartbeat = std::time::Instant::now();
                match famshield_session::keyboard_monitor::send_heartbeat_via_ipc(
                    session_id,
                    current_proc,
                    is_idle,
                    idle_ms.into(),
                )
                .await
                {
                    Ok(Some(famshield_service::ipc::IpcMessage::CaptureEvidenceCommand {
                        event_id,
                        trigger,
                    })) => {
                        info!(
                            "Received CaptureEvidenceCommand from service for trigger: '{}'",
                            trigger
                        );

                        // 1. Show notification toast to user
                        let tray_hwnd = SystemTray::get_tray_hwnd();
                        famshield_session::FriendlyToast::notify_content_blocked(tray_hwnd, &trigger);

                        // 2. Capture blurred screenshot in RAM and submit to service
                        tokio::spawn(async move {
                            let verifier = famshield_session::SmartVisualVerifier::new(15);
                            let capture_res = tokio::task::spawn_blocking(move || {
                                verifier.capture_blurred_frame()
                            })
                            .await;

                            match capture_res {
                                Ok(Ok(jpeg_bytes)) => {
                                    if let Err(e) =
                                        famshield_session::keyboard_monitor::send_visual_evidence_via_ipc(
                                            event_id,
                                            trigger,
                                            jpeg_bytes,
                                        )
                                        .await
                                    {
                                        tracing::warn!("Failed to submit web block visual evidence: {}", e);
                                    }
                                }
                                Ok(Err(e)) => {
                                    tracing::warn!("Failed to capture web block evidence frame: {}", e);
                                }
                                Err(e) => {
                                    tracing::warn!("Spawn blocking error: {}", e);
                                }
                            }
                        });
                    }
                    Ok(_) => {}
                    Err(e) => {
                        tracing::debug!("Heartbeat IPC: {}", e);
                    }
                }
            }
        }
    });

    // 5. Keep session helper active until terminated
    info!("Session Helper initialized and active.");
    tokio::signal::ctrl_c().await?;
    info!("Session Helper terminating.");
    Ok(())
}

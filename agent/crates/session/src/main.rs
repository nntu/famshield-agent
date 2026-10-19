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

    // 4. Periodic foreground and idle tracking loop (every 1 second)
    let km_for_tracker = keyboard_monitor.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));
        loop {
            interval.tick().await;
            let idle_ms = ForegroundTracker::get_system_idle_millis();
            let is_idle = idle_ms >= 180_000; // 3 minutes

            if let Some(proc_name) = ForegroundTracker::get_foreground_process_name() {
                if let Ok(exe) = famshield_core::domain::ExecutableName::parse(&proc_name) {
                    tracker.switch_active_window(exe, std::time::Instant::now());
                    km_for_tracker.set_active_app(&proc_name);
                }
            }
            tracker.tick_usage(1, is_idle);
        }
    });

    // 5. Keep session helper active until terminated
    info!("Session Helper initialized and active.");
    tokio::signal::ctrl_c().await?;
    info!("Session Helper terminating.");
    Ok(())
}

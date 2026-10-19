#![windows_subsystem = "windows"]

use famshield_session::{ForegroundTracker, PinDialog, SystemTray};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
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

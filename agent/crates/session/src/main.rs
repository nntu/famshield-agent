use tracing::info;
use famshield_session::{ForegroundTracker, PinDialog, SystemTray};

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

    // 1. Start foreground app monitor
    let _tracker = ForegroundTracker::new(300);

    // 2. Start lightweight System Tray with context menu & toast support
    SystemTray::start();
    info!("System Tray icon active at taskbar notification area.");

    // 3. Keep session helper active until terminated
    info!("Session Helper initialized and active.");
    tokio::signal::ctrl_c().await?;
    info!("Session Helper terminating.");
    Ok(())
}

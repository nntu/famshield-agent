use famshield_service::ServiceController;
use famshield_storage::StorageEngine;
use std::path::PathBuf;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // Store database under ProgramData on Windows
    let data_dir = std::env::var("ProgramData")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("FamShield");
    let _ = std::fs::create_dir_all(&data_dir);
    let db_path = data_dir.join("famshield.db");

    let storage = StorageEngine::open(&db_path)?;

    // Handle command-line elevated Master PIN reset
    let args: Vec<String> = std::env::args().collect();
    if let Some(pos) = args.iter().position(|a| a == "--reset-pin") {
        if let Some(new_pin) = args.get(pos + 1) {
            println!("==> [FamShield Administrator Elevation] Resetting Master PIN...");
            match famshield_local_admin::MasterPinAuthenticator::reset_pin_elevated(
                &storage, new_pin,
            ) {
                Ok(()) => {
                    println!("[FamShield] SUCCESS: Master PIN has been successfully reset!");
                    return Ok(());
                }
                Err(e) => {
                    eprintln!("[FamShield] ERROR: Failed to reset Master PIN: {}", e);
                    std::process::exit(1);
                }
            }
        } else {
            eprintln!("[FamShield] ERROR: Missing new PIN. Usage: famshield-service.exe --reset-pin <NEW_PIN>");
            std::process::exit(1);
        }
    }

    info!("Starting FamShield Service runtime...");
    let controller = ServiceController::new(storage);

    let (shutdown_tx, shutdown_rx) = tokio::sync::broadcast::channel(1);

    // Trap Ctrl+C or service stop
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        info!("Termination signal detected, initiating clean shutdown...");
        let _ = shutdown_tx.send(());
    });

    controller.run(shutdown_rx).await;
    info!("FamShield Service stopped cleanly.");
    Ok(())
}

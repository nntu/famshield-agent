use std::path::PathBuf;
use tracing::info;
use childprotect_service::ServiceController;
use childprotect_storage::StorageEngine;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    info!("Starting ChildProtect Service runtime...");

    // Store database under ProgramData on Windows
    let data_dir = std::env::var("ProgramData")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("ChildProtect");
    let _ = std::fs::create_dir_all(&data_dir);
    let db_path = data_dir.join("childprotect.db");

    let storage = StorageEngine::open(&db_path)?;
    let controller = ServiceController::new(storage);

    let (shutdown_tx, shutdown_rx) = tokio::sync::broadcast::channel(1);

    // Trap Ctrl+C or service stop
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        info!("Termination signal detected, initiating clean shutdown...");
        let _ = shutdown_tx.send(());
    });

    controller.run(shutdown_rx).await;
    info!("ChildProtect Service stopped cleanly.");
    Ok(())
}

use tracing::info;
use famshield_session::ForegroundTracker;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    info!("Starting FamShield Interactive Session Helper...");

    let _tracker = ForegroundTracker::new(300);

    // Runs session loop monitoring active foreground window
    info!("Session Helper initialized and active.");
    tokio::signal::ctrl_c().await?;
    info!("Session Helper terminating.");
    Ok(())
}

use famshield_service::ServiceController;
use famshield_storage::StorageEngine;
use std::path::PathBuf;
use tracing::{debug, info};

pub const SERVICE_NAME: &str = "FamShieldService";

fn get_data_dir() -> PathBuf {
    let data_dir = std::env::var("ProgramData")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("FamShield");
    let _ = std::fs::create_dir_all(&data_dir);
    data_dir
}

fn handle_reset_pin(args: &[String]) -> Result<bool, Box<dyn std::error::Error>> {
    if let Some(pos) = args.iter().position(|a| a == "--reset-pin") {
        if let Some(new_pin) = args.get(pos + 1) {
            println!("==> [FamShield Administrator Elevation] Resetting Master PIN...");
            let db_path = get_data_dir().join("famshield.db");
            let storage = StorageEngine::open(&db_path)?;
            match famshield_local_admin::MasterPinAuthenticator::reset_pin_elevated(&storage, new_pin) {
                Ok(()) => {
                    println!("[FamShield] SUCCESS: Master PIN has been successfully reset!");
                    return Ok(true);
                }
                Err(e) => {
                    eprintln!("[FamShield] ERROR: Failed to reset Master PIN: {}", e);
                    std::process::exit(1);
                }
            }
        } else {
            eprintln!(
                "[FamShield] ERROR: Missing new PIN. Usage: famshield-service.exe --reset-pin <NEW_PIN>"
            );
            std::process::exit(1);
        }
    }
    Ok(false)
}

#[cfg(windows)]
windows_service::define_windows_service!(ffi_service_main, my_service_main);

#[cfg(windows)]
fn my_service_main(args: Vec<std::ffi::OsString>) {
    if let Err(e) = service_main_entry(args) {
        tracing::error!("FamShield service execution failed: {:?}", e);
    }
}

#[cfg(windows)]
fn run_windows_service() -> Result<(), windows_service::Error> {
    use windows_service::service_dispatcher;
    service_dispatcher::start(SERVICE_NAME, ffi_service_main)
}

#[cfg(windows)]
fn service_main_entry(_args: Vec<std::ffi::OsString>) -> Result<(), Box<dyn std::error::Error>> {
    use std::time::Duration;
    use windows_service::service::{
        ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
        ServiceType,
    };
    use windows_service::service_control_handler::{self, ServiceControlHandlerResult};

    let rt = tokio::runtime::Runtime::new()?;
    let (shutdown_tx, shutdown_rx) = tokio::sync::broadcast::channel(1);
    let shutdown_tx_clone = shutdown_tx.clone();

    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Stop | ServiceControl::Shutdown => {
                info!("Windows SCM requested service stop/shutdown");
                let _ = shutdown_tx_clone.send(());
                ServiceControlHandlerResult::NoError
            }
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)?;

    // Report RUNNING status to SCM
    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP | ServiceControlAccept::SHUTDOWN,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    // Execute ServiceController in Tokio runtime
    rt.block_on(async {
        let db_path = get_data_dir().join("famshield.db");
        match StorageEngine::open(&db_path) {
            Ok(storage) => {
                let controller = ServiceController::new(storage);
                controller.run(shutdown_rx).await;
            }
            Err(e) => {
                tracing::error!("Failed to initialize database: {}", e);
            }
        }
    });

    // Report STOPPED status to SCM upon clean exit
    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Stopped,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    Ok(())
}

fn run_console() -> Result<(), Box<dyn std::error::Error>> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        info!("Starting FamShield Service in interactive console mode...");
        let db_path = get_data_dir().join("famshield.db");
        let storage = StorageEngine::open(&db_path)?;
        let controller = ServiceController::new(storage);

        let (shutdown_tx, shutdown_rx) = tokio::sync::broadcast::channel(1);
        tokio::spawn(async move {
            tokio::signal::ctrl_c().await.ok();
            info!("Ctrl+C signal received, shutting down gracefully...");
            let _ = shutdown_tx.send(());
        });

        controller.run(shutdown_rx).await;
        info!("FamShield Service console execution finished.");
        Ok(())
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let args: Vec<String> = std::env::args().collect();
    if handle_reset_pin(&args)? {
        return Ok(());
    }

    let is_console = args.iter().any(|a| a == "--console" || a == "-c" || a == "run-console");

    #[cfg(windows)]
    if !is_console {
        // Attempt starting under Windows Service Control Manager (SCM)
        if let Err(e) = run_windows_service() {
            debug!("SCM connect returned: {}. Falling back to console mode.", e);
            return run_console();
        }
        return Ok(());
    }

    run_console()
}

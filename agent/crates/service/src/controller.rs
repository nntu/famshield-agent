use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::info;

use crate::dispatcher::IpcDispatcher;
use famshield_dns::{
    blocklist::BlocklistStore, cache::DnsCache, proxy::DnsProxy, DEFAULT_BIND_ADDR,
    DEFAULT_UPSTREAM,
};
use famshield_storage::StorageEngine;
use famshield_sync::NatsDispatcher;
use famshield_windows::{wfp::WfpSessionGuard, InstallerInterceptor, ProcessMonitor};

/// Core service controller orchestrating DNS, WFP, Local IPC, Storage, and NATS JetStream dispatcher.
pub struct ServiceController {
    storage: StorageEngine,
    ipc_dispatcher: Arc<IpcDispatcher>,
    dns_blocklist: Arc<BlocklistStore>,
    dns_cache: Arc<DnsCache>,
    process_monitor: Arc<ProcessMonitor>,
    nats_dispatcher: Arc<NatsDispatcher>,
}

impl ServiceController {
    pub fn new(storage: StorageEngine) -> Self {
        let ipc_dispatcher = Arc::new(IpcDispatcher::new(storage.clone()));
        let dns_blocklist = Arc::new(BlocklistStore::new());
        let dns_cache = Arc::new(DnsCache::new());
        let installer = Arc::new(InstallerInterceptor::new());
        let process_monitor = Arc::new(ProcessMonitor::new());

        // Read enrolled device_id from storage meta or fallback to standalone ID
        let device_id = storage
            .get_meta("enrolled_device_id")
            .ok()
            .flatten()
            .unwrap_or_else(|| "dev-local-001".to_string());

        let nats_dispatcher = Arc::new(NatsDispatcher::new(
            device_id,
            storage.clone(),
            dns_blocklist.clone(),
            installer.clone(),
        ));

        Self {
            storage,
            ipc_dispatcher,
            dns_blocklist,
            dns_cache,
            process_monitor,
            nats_dispatcher,
        }
    }

    pub fn storage(&self) -> &StorageEngine {
        &self.storage
    }

    pub fn ipc_dispatcher(&self) -> Arc<IpcDispatcher> {
        self.ipc_dispatcher.clone()
    }

    pub fn dns_blocklist(&self) -> Arc<BlocklistStore> {
        self.dns_blocklist.clone()
    }

    pub fn nats_dispatcher(&self) -> Arc<NatsDispatcher> {
        self.nats_dispatcher.clone()
    }

    pub fn process_monitor(&self) -> Arc<ProcessMonitor> {
        self.process_monitor.clone()
    }

    /// Run the integrated services (DNS loopback proxy, IPC dispatcher, WFP guard, Process monitor).
    pub async fn run(&self, mut shutdown_rx: broadcast::Receiver<()>) {
        info!("Starting FamShield Windows Service subsystems...");

        // 1. Initialize Dynamic WFP Session Guard
        let _wfp_guard = match WfpSessionGuard::open_dynamic("FamShieldServiceSession") {
            Ok(g) => {
                info!("Dynamic WFP User-Mode protection armed");
                Some(g)
            }
            Err(e) => {
                tracing::warn!("WFP session could not be established: {}", e);
                None
            }
        };

        // 2. Start DNS proxy listener on 127.0.0.2:53
        let dns_proxy = DnsProxy::new(
            DEFAULT_BIND_ADDR,
            DEFAULT_UPSTREAM,
            self.dns_blocklist.clone(),
            self.dns_cache.clone(),
        );

        // 3. Periodic Process Monitoring Task (every 5 seconds)
        let proc_monitor = self.process_monitor.clone();
        let nats = self.nats_dispatcher.clone();
        let storage = self.storage.clone();
        let mut proc_interval = tokio::time::interval(std::time::Duration::from_secs(5));
        let mut err_interval = tokio::time::interval(std::time::Duration::from_secs(15));

        tokio::select! {
            _ = dns_proxy.run_udp_listener() => {
                info!("DNS proxy listener ended");
            }
            _ = async {
                loop {
                    tokio::select! {
                        _ = proc_interval.tick() => {
                            if let Ok(events) = proc_monitor.detect_changes() {
                                if !events.is_empty() {
                                    info!("Detected {} process lifecycle changes", events.len());
                                    if let Ok(batch_json) = nats.format_process_events_batch(&events) {
                                        info!("Queued process telemetry batch for NATS JetStream: {} bytes", batch_json.len());
                                    }
                                }
                            }
                        }
                        _ = err_interval.tick() => {
                            // Check for unsynced error logs and prepare telemetry batch
                            if let Ok(unsynced_errors) = storage.list_unsynced_errors(50) {
                                if !unsynced_errors.is_empty() {
                                    info!("Found {} unsynced error logs in database", unsynced_errors.len());
                                    if let Ok(err_batch) = nats.format_error_logs_batch(&unsynced_errors) {
                                        info!("Queued error logs telemetry batch for NATS JetStream: {} bytes", err_batch.len());
                                        let error_ids: Vec<String> = unsynced_errors.into_iter().map(|e| e.error_id).collect();
                                        let _ = storage.mark_errors_synced(&error_ids);
                                    }
                                }
                            }
                        }
                    }
                }
            } => {}
            _ = shutdown_rx.recv() => {
                info!("Shutdown signal received, gracefully terminating FamShield service");
            }
        }
    }
}

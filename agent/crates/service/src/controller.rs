use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::info;

use crate::dispatcher::IpcDispatcher;
use famshield_dns::{
    blocklist::BlocklistStore, cache::DnsCache, proxy::DnsProxy, DEFAULT_BIND_ADDR,
    DEFAULT_UPSTREAM,
};
use famshield_storage::StorageEngine;
use famshield_sync::{
    CloudSyncClient, NatsDispatcher, TelemetryEvent, TelemetryPipeline, TelemetrySyncWorker,
};
use famshield_windows::{wfp::WfpSessionGuard, InstallerInterceptor, ProcessMonitor};

/// Core service controller orchestrating DNS, WFP, Local IPC, Storage, Telemetry Sync, and NATS JetStream dispatcher.
pub struct ServiceController {
    storage: StorageEngine,
    ipc_dispatcher: Arc<IpcDispatcher>,
    dns_blocklist: Arc<BlocklistStore>,
    dns_cache: Arc<DnsCache>,
    process_monitor: Arc<ProcessMonitor>,
    nats_dispatcher: Arc<NatsDispatcher>,
    local_admin_server: Arc<famshield_local_admin::LocalAdminServer>,
    telemetry_pipeline: Arc<TelemetryPipeline>,
    sync_worker: Arc<TelemetrySyncWorker>,
}

impl ServiceController {
    pub fn new(storage: StorageEngine) -> Self {
        let ipc_dispatcher = Arc::new(IpcDispatcher::new(storage.clone()));
        let dns_blocklist = Arc::new(BlocklistStore::new());
        let dns_cache = Arc::new(DnsCache::new());
        let installer = Arc::new(InstallerInterceptor::new());
        let process_monitor = Arc::new(ProcessMonitor::new());

        // Pre-populate DNS blocklist from persistent local rules in SQLite
        if let Ok(rules) = storage.list_local_rules() {
            let domain_rules: Vec<String> = rules
                .into_iter()
                .filter(|r| r.rule_type == "domain" && r.enabled)
                .map(|r| r.rule_id)
                .collect();
            if !domain_rules.is_empty() {
                dns_blocklist.replace_blocklist(domain_rules);
            }
        }

        // Read enrolled device_id from storage meta or fallback to standalone ID
        let device_id = storage
            .get_meta("enrolled_device_id")
            .ok()
            .flatten()
            .unwrap_or_else(|| "dev-local-001".to_string());

        let nats_dispatcher = Arc::new(NatsDispatcher::new(
            device_id.clone(),
            storage.clone(),
            dns_blocklist.clone(),
            installer.clone(),
        ));

        let local_admin_svc = Arc::new(famshield_local_admin::LocalAdminService::new(
            storage.clone(),
        ));
        let local_admin_server = Arc::new(famshield_local_admin::LocalAdminServer::new(
            local_admin_svc,
            Some(dns_blocklist.clone()),
        ));

        let telemetry_pipeline = Arc::new(TelemetryPipeline::new(device_id, storage.clone()));

        let server_url = storage
            .get_meta("server_url")
            .ok()
            .flatten()
            .unwrap_or_else(|| "http://127.0.0.1:8080".to_string());
        let cloud_client = Arc::new(CloudSyncClient::new(server_url));

        let sync_worker = Arc::new(TelemetrySyncWorker::new(
            telemetry_pipeline.clone(),
            cloud_client,
            storage.clone(),
            Some(dns_blocklist.clone()),
        ));

        Self {
            storage,
            ipc_dispatcher,
            dns_blocklist,
            dns_cache,
            process_monitor,
            nats_dispatcher,
            local_admin_server,
            telemetry_pipeline,
            sync_worker,
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

    pub fn local_admin_server(&self) -> Arc<famshield_local_admin::LocalAdminServer> {
        self.local_admin_server.clone()
    }

    /// Run the integrated services (DNS loopback proxy, IPC dispatcher, WFP guard, Process monitor).
    pub async fn run(&self, mut shutdown_rx: broadcast::Receiver<()>) {
        info!("Starting FamShield Windows Service subsystems...");

        // 1. Initialize Dynamic WFP Session Guard & enforce Port 53 DNS redirection
        let _wfp_guard = match WfpSessionGuard::open_dynamic("FamShieldServiceSession") {
            Ok(mut g) => {
                info!("Dynamic WFP User-Mode protection armed");
                let upstream_ips: Vec<std::net::Ipv4Addr> = DEFAULT_UPSTREAM
                    .parse::<std::net::SocketAddr>()
                    .ok()
                    .and_then(|addr| match addr {
                        std::net::SocketAddr::V4(v4) => Some(*v4.ip()),
                        _ => None,
                    })
                    .into_iter()
                    .collect();

                if let Err(e) = g.block_external_dns(&upstream_ips) {
                    tracing::warn!("Failed to install WFP Port 53 block filters: {}", e);
                }
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

        // 3. Start Secure Named Pipe Server (Session 0 <-> User Session)
        let ipc_server = crate::ipc::NamedPipeServer::new(self.ipc_dispatcher.clone());
        let ipc_shutdown_rx = shutdown_rx.resubscribe();
        tokio::spawn(async move {
            if let Err(e) = ipc_server.run(ipc_shutdown_rx).await {
                tracing::warn!("Named Pipe IPC server encountered error: {}", e);
            }
        });

        // 4. Start Local Admin Web Server (http://127.0.0.1:48123)
        let admin_server = self.local_admin_server.clone();
        let admin_shutdown_rx = shutdown_rx.resubscribe();
        tokio::spawn(async move {
            if let Err(e) = admin_server.run("127.0.0.1:48123", admin_shutdown_rx).await {
                tracing::warn!("Local Admin Web Server encountered error: {}", e);
            }
        });

        // 5. Start Telemetry & Cloud Sync Worker (HTTP Outbox Batch + Heartbeat + Dynamic Policy Sync)
        let sync_worker = self.sync_worker.clone();
        let sync_shutdown_rx = shutdown_rx.resubscribe();
        tokio::spawn(async move {
            sync_worker.run(sync_shutdown_rx).await;
        });

        // 6. Periodic Process Monitoring Task (every 5 seconds)
        let proc_monitor = self.process_monitor.clone();
        let nats = self.nats_dispatcher.clone();
        let storage = self.storage.clone();
        let admin_server_proc = self.local_admin_server.clone();
        let telemetry_pipeline = self.telemetry_pipeline.clone();
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
                                    for ev in &events {
                                        if ev.event_type == "process_started" {
                                            admin_server_proc.set_active_app(&ev.process_name).await;
                                        }
                                        let _ = telemetry_pipeline.queue_event(&TelemetryEvent::new(
                                            format!("proc-{}", chrono::Utc::now().timestamp_micros()),
                                            "process_event",
                                            chrono::Utc::now(),
                                            serde_json::to_value(ev).unwrap_or_default(),
                                        ));
                                    }
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
                                    for err_rec in &unsynced_errors {
                                        let _ = telemetry_pipeline.queue_event(&TelemetryEvent::new(
                                            format!("err-{}", err_rec.error_id),
                                            "error_log",
                                            chrono::Utc::now(),
                                            serde_json::to_value(err_rec).unwrap_or_default(),
                                        ));
                                    }
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

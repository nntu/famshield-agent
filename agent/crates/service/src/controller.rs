use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{info, warn};

use crate::dispatcher::IpcDispatcher;
use famshield_dns::{
    blocklist::BlocklistStore, cache::DnsCache, proxy::DnsProxy, DEFAULT_BIND_ADDR,
    DEFAULT_UPSTREAM,
};
use famshield_storage::StorageEngine;
use famshield_sync::{
    load_command_signing_key, CloudSyncClient, NatsConnectionWorker, NatsDispatcher,
    TelemetryEvent, TelemetryPipeline, TelemetrySyncWorker,
};
use famshield_windows::{wfp::WfpSessionGuard, InstallerInterceptor, ProcessMonitor};

pub const DEFAULT_ADMIN_PORT: u16 = 38123;
pub const FALLBACK_ADMIN_PORTS: &[u16] = &[38123, 18123, 28123, 48123];

pub fn get_data_dir() -> std::path::PathBuf {
    let data_dir = std::env::var("ProgramData")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join("FamShield");
    let _ = std::fs::create_dir_all(&data_dir);
    data_dir
}

pub fn save_active_admin_port(port: u16) {
    let port_file = get_data_dir().join("admin_port.txt");
    let _ = std::fs::write(port_file, port.to_string());
}

pub fn get_active_admin_port() -> u16 {
    let port_file = get_data_dir().join("admin_port.txt");
    if let Ok(content) = std::fs::read_to_string(port_file) {
        if let Ok(port) = content.trim().parse::<u16>() {
            return port;
        }
    }
    DEFAULT_ADMIN_PORT
}

/// Core service controller orchestrating DNS, WFP, Local IPC, Storage, Telemetry Sync, and NATS JetStream dispatcher.
pub struct ServiceController {
    storage: StorageEngine,
    ipc_dispatcher: Arc<IpcDispatcher>,
    dns_blocklist: Arc<BlocklistStore>,
    dns_cache: Arc<DnsCache>,
    dns_balancer: Arc<famshield_dns::NumaDnsBalancer>,
    process_monitor: Arc<ProcessMonitor>,
    nats_dispatcher: Arc<NatsDispatcher>,
    local_admin_server: Arc<famshield_local_admin::LocalAdminServer>,
    telemetry_pipeline: Arc<TelemetryPipeline>,
    sync_worker: Arc<TelemetrySyncWorker>,
    nats_worker: Arc<NatsConnectionWorker>,
    is_updating: Arc<std::sync::atomic::AtomicBool>,
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

        let mut nats_dispatcher = NatsDispatcher::new(
            device_id.clone(),
            storage.clone(),
            dns_blocklist.clone(),
            installer.clone(),
        );
        match load_command_signing_key(&storage) {
            Ok(Some(signing_key)) => {
                nats_dispatcher = nats_dispatcher.with_signing_key(signing_key);
            }
            Ok(None) => {}
            Err(error) => {
                warn!("Unable to load remote command signing key: {}", error);
            }
        }
        let nats_dispatcher = Arc::new(nats_dispatcher);

        let configured_upstream = storage
            .get_meta("dns_upstream")
            .ok()
            .flatten()
            .unwrap_or_else(|| "1.1.1.1".to_string());

        let upstreams = match famshield_windows::NrptManager::get_system_dns_servers() {
            detected if !detected.is_empty() => {
                let mut list = vec![configured_upstream.clone()];
                for d in detected {
                    if !list.contains(&d) {
                        list.push(d);
                    }
                }
                list
            }
            _ => vec![
                configured_upstream.clone(),
                "8.8.8.8".to_string(),
                "9.9.9.9".to_string(),
            ],
        };
        let dns_balancer = Arc::new(famshield_dns::NumaDnsBalancer::new(&upstreams));

        let is_updating = Arc::new(std::sync::atomic::AtomicBool::new(false));

        let local_admin_svc = Arc::new(famshield_local_admin::LocalAdminService::new(
            storage.clone(),
        ));
        let local_admin_server = Arc::new(
            famshield_local_admin::LocalAdminServer::new(
                local_admin_svc,
                Some(dns_blocklist.clone()),
            )
            .with_dns_cache(dns_cache.clone())
            .with_dns_balancer(dns_balancer.clone())
            .with_terminating_flag(is_updating.clone()),
        );

        let telemetry_pipeline = Arc::new(TelemetryPipeline::new(&device_id, storage.clone()));
        ipc_dispatcher.set_telemetry(telemetry_pipeline.clone());

        let data_dir = get_data_dir();
        let server_url_file = data_dir.join("server_url.txt");
        let bootstrap_file = data_dir.join("bootstrap.json");

        let mut server_url = storage
            .get_meta("server_url")
            .ok()
            .flatten()
            .unwrap_or_default();

        if server_url.is_empty() {
            if let Ok(content) = std::fs::read_to_string(&server_url_file) {
                let trimmed = content.trim().to_string();
                if !trimmed.is_empty() {
                    server_url = trimmed.clone();
                    let _ = storage.set_meta("server_url", &trimmed);
                }
            }
        }

        if bootstrap_file.exists() {
            if let Ok(content) = std::fs::read_to_string(&bootstrap_file) {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(url) = val.get("server_url").and_then(|v| v.as_str()) {
                        let trimmed = url.trim().to_string();
                        if !trimmed.is_empty() {
                            server_url = trimmed.clone();
                            let _ = storage.set_meta("server_url", &trimmed);
                        }
                    }
                    if let Some(code) = val.get("enrollment_code").and_then(|v| v.as_str()) {
                        let trimmed = code.trim().to_string();
                        if trimmed.len() == 6 {
                            let _ = storage.set_meta("enrollment_code", &trimmed);
                        }
                    }
                }
            }
            let _ = std::fs::rename(&bootstrap_file, data_dir.join("bootstrap.json.done"));
        }

        if server_url.is_empty() {
            server_url = "http://127.0.0.1:8080".to_string();
        }
        let cloud_client = Arc::new(CloudSyncClient::new(&server_url));

        let sync_worker = Arc::new(TelemetrySyncWorker::new(
            telemetry_pipeline.clone(),
            cloud_client,
            storage.clone(),
            Some(dns_blocklist.clone()),
        ));

        let nats_url = storage
            .get_meta("nats_url")
            .ok()
            .flatten()
            .unwrap_or_else(|| {
                if server_url.starts_with("https://") {
                    let host = server_url
                        .trim_start_matches("https://")
                        .trim_end_matches('/');
                    format!("wss://{}/nats", host)
                } else if server_url.starts_with("http://") {
                    let host = server_url
                        .trim_start_matches("http://")
                        .split(':')
                        .next()
                        .unwrap_or("127.0.0.1");
                    format!("ws://{}:9222", host)
                } else {
                    "ws://127.0.0.1:9222".to_string()
                }
            });

        let nats_worker = Arc::new(NatsConnectionWorker::new(
            device_id,
            nats_url,
            nats_dispatcher.clone(),
            storage.clone(),
        ));

        Self {
            storage,
            ipc_dispatcher,
            dns_blocklist,
            dns_cache,
            dns_balancer,
            process_monitor,
            nats_dispatcher,
            local_admin_server,
            telemetry_pipeline,
            sync_worker,
            nats_worker,
            is_updating,
        }
    }

    pub fn storage(&self) -> &StorageEngine {
        &self.storage
    }

    pub fn ipc_dispatcher(&self) -> Arc<IpcDispatcher> {
        self.ipc_dispatcher.clone()
    }

    pub fn is_updating(&self) -> Arc<std::sync::atomic::AtomicBool> {
        self.is_updating.clone()
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

        // 0. Update Watchdog check: ensure we didn't crash on previous boot following an update
        let app_data = std::env::var("ProgramData")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .join("FamShield");
        let install_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| std::path::PathBuf::from(r"C:\Program Files\FamShield"));

        let atomic_updater = famshield_updater::WindowsAtomicUpdater::new(
            env!("CARGO_PKG_VERSION"),
            &app_data,
            &install_dir,
        );

        match atomic_updater.check_and_recover_health() {
            Ok(famshield_updater::WatchdogStatus::PendingVerification {
                target_version,
                boot_attempts,
            }) => {
                tracing::warn!(
                    "WATCHDOG: Newly updated v{} running in observation window (attempt {}). 60s timer started.",
                    target_version, boot_attempts
                );
                let updater_for_confirm = famshield_updater::WindowsAtomicUpdater::new(
                    env!("CARGO_PKG_VERSION"),
                    &app_data,
                    &install_dir,
                );
                tokio::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                    if let Err(e) = updater_for_confirm.confirm_update_healthy() {
                        tracing::error!("Watchdog: Failed to confirm update health: {}", e);
                    } else {
                        tracing::info!(
                            "WATCHDOG: Update v{} confirmed healthy! Observation window closed.",
                            target_version
                        );
                    }
                });
            }
            Ok(famshield_updater::WatchdogStatus::RolledBack {
                failed_version,
                restored_version,
            }) => {
                tracing::error!(
                    "WATCHDOG: Previous boot crashed after updating to v{}! Automated rollback restored v{}.",
                    failed_version, restored_version
                );
            }
            Ok(famshield_updater::WatchdogStatus::Normal)
            | Ok(famshield_updater::WatchdogStatus::Verified) => {
                tracing::debug!("Watchdog status: Healthy/Normal");
            }
            Err(e) => {
                tracing::warn!("Watchdog check encountered error: {}", e);
            }
        }

        // Resolve initial upstream DNS servers: user-configured upstream + detected system DNS
        let configured_upstream = self
            .storage
            .get_meta("dns_upstream")
            .ok()
            .flatten()
            .unwrap_or_else(|| DEFAULT_UPSTREAM.to_string());

        let system_dns = famshield_windows::NrptManager::get_system_dns_servers();
        let mut upstreams = vec![configured_upstream.clone()];
        for s in system_dns {
            let addr = if s.contains(':') {
                s
            } else {
                format!("{}:53", s)
            };
            if !upstreams.contains(&addr) {
                upstreams.push(addr);
            }
        }
        tracing::info!(
            "Initializing DNS Proxy with upstream cascade: {:?}",
            upstreams
        );

        // 1. Initialize Dynamic WFP Session Guard & enforce Port 53 DNS redirection
        let _wfp_guard = match WfpSessionGuard::open_dynamic("FamShieldServiceSession") {
            Ok(mut g) => {
                info!("Dynamic WFP User-Mode protection armed");
                let mut upstream_ips: Vec<std::net::Ipv4Addr> = Vec::new();
                for u in &upstreams {
                    if let Ok(std::net::SocketAddr::V4(v4)) = u.parse::<std::net::SocketAddr>() {
                        if !upstream_ips.contains(v4.ip()) {
                            upstream_ips.push(*v4.ip());
                        }
                    }
                }

                if let Err(e) = g.block_external_dns(&upstream_ips) {
                    tracing::warn!("Failed to install WFP Port 53 block filters: {}", e);
                }
                if let Err(e) = g.block_bypass_vpn_ports() {
                    tracing::warn!("Failed to install WFP anti-bypass VPN port filters: {}", e);
                }
                Some(g)
            }
            Err(e) => {
                tracing::warn!("WFP session could not be established: {}", e);
                None
            }
        };

        // Apply NRPT rule to route root namespace '.' to FamShield local DNS proxy (127.0.0.2:53)
        if let Err(e) = famshield_windows::NrptManager::apply_nrpt_rule("127.0.0.2") {
            tracing::warn!("Failed to apply NRPT DNS rule: {}", e);
        }

        // 2. Start DNS proxy listener on 127.0.0.2:53 with real-time query logging
        let (dns_log_tx, mut dns_log_rx) =
            tokio::sync::mpsc::unbounded_channel::<famshield_dns::DnsLogEntry>();

        let dns_proxy = DnsProxy::new(
            DEFAULT_BIND_ADDR,
            configured_upstream,
            self.dns_blocklist.clone(),
            self.dns_cache.clone(),
        )
        .with_balancer(self.dns_balancer.clone())
        .with_log_sender(dns_log_tx);

        let dns_storage = self.storage.clone();
        let dns_telemetry = self.telemetry_pipeline.clone();
        let dns_dispatcher = self.ipc_dispatcher.clone();
        tokio::spawn(async move {
            let mut batch = Vec::with_capacity(32);
            let mut interval = tokio::time::interval(std::time::Duration::from_millis(500));
            loop {
                tokio::select! {
                    Some(entry) = dns_log_rx.recv() => {
                        let now = chrono::Utc::now();
                        let record = famshield_storage::QueryLogRecord {
                            domain: entry.domain.clone(),
                            qtype: entry.qtype.clone(),
                            action: entry.action.clone(),
                            reason: entry.reason.clone(),
                            rule_id: entry.rule_id.clone(),
                            occurred_at: now,
                        };
                        batch.push(record);
                        if batch.len() >= 32 {
                            let _ = dns_storage.insert_query_logs_batch(&batch);
                            batch.clear();
                        }

                        if entry.action == "BLOCK" {
                            let event_id = format!("dns-block-{}", now.timestamp_micros());
                            let alert_payload = serde_json::json!({
                                "domain": entry.domain,
                                "qtype": entry.qtype,
                                "action": entry.action,
                                "reason": entry.reason,
                                "timestamp": now.to_rfc3339(),
                            });
                            let _ = dns_telemetry.queue_event(&TelemetryEvent::new(
                                event_id.clone(),
                                "dns_query",
                                now,
                                alert_payload,
                            ));

                            // Queue visual evidence capture trigger for user session helper
                            let trigger_desc = format!("WEB_BLOCK: {} ({})", entry.domain, entry.reason);
                            dns_dispatcher.queue_capture_trigger(event_id, trigger_desc);
                        }
                    }
                    _ = interval.tick() => {
                        if !batch.is_empty() {
                            let _ = dns_storage.insert_query_logs_batch(&batch);
                            batch.clear();
                        }
                    }
                }
            }
        });

        // 3. Start Secure Named Pipe Server (Session 0 <-> User Session)
        let ipc_server = crate::ipc::NamedPipeServer::new(self.ipc_dispatcher.clone());
        let ipc_shutdown_rx = shutdown_rx.resubscribe();
        tokio::spawn(async move {
            if let Err(e) = ipc_server.run(ipc_shutdown_rx).await {
                tracing::warn!("Named Pipe IPC server encountered error: {}", e);
            }
        });

        // 4. Start Local Admin Web Server (http://127.0.0.1:38123 with fallback)
        let admin_server = self.local_admin_server.clone();
        let admin_shutdown_rx = shutdown_rx.resubscribe();
        let default_addr = format!("127.0.0.1:{}", DEFAULT_ADMIN_PORT);
        tokio::spawn(async move {
            if let Err(e) = admin_server.run(&default_addr, admin_shutdown_rx).await {
                tracing::warn!("Local Admin Web Server encountered error: {}", e);
            }
        });

        // 5. Start Telemetry & Cloud Sync Worker (HTTP Outbox Batch + Heartbeat + Dynamic Policy Sync)
        let sync_worker = self.sync_worker.clone();
        let sync_shutdown_rx = shutdown_rx.resubscribe();
        tokio::spawn(async move {
            sync_worker.run(sync_shutdown_rx).await;
        });

        // 5b. Start NATS Real-Time Control Plane Worker (Duplex Push over Cloudflare WebSocket WSS / TCP)
        let nats_worker = self.nats_worker.clone();
        let nats_shutdown_rx = shutdown_rx.resubscribe();
        tokio::spawn(async move {
            nats_worker.run(nats_shutdown_rx).await;
        });

        // 6. Start Anti-Tampering & Self-Healing Watchdog (Session 0 auto-respawn + NRPT drift lock)
        let watchdog = crate::watchdog::SystemWatchdog::new(self.is_updating.clone())
            .with_telemetry(self.telemetry_pipeline.clone());
        let watchdog_shutdown_rx = shutdown_rx.resubscribe();
        tokio::spawn(async move {
            watchdog.run(watchdog_shutdown_rx).await;
        });

        // 7. Periodic Storage Maintenance Task: Evidence Purge & DNS Log Retention Rollup (every 6 hours)
        let storage_maint = self.storage.clone();
        tokio::spawn(async move {
            let mut maint_interval =
                tokio::time::interval(std::time::Duration::from_secs(6 * 3600));
            maint_interval.tick().await;
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            loop {
                let now = chrono::Utc::now();
                let _ = storage_maint.purge_expired_evidence(now);

                let auto_prune = storage_maint
                    .get_meta("dns_auto_prune_enabled")
                    .ok()
                    .flatten()
                    .map(|v| v != "false")
                    .unwrap_or(true);

                if auto_prune {
                    let retention_days: i64 = storage_maint
                        .get_meta("dns_log_retention_days")
                        .ok()
                        .flatten()
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(7);

                    let cutoff = now - chrono::Duration::days(retention_days);
                    if let Ok(deleted) = storage_maint.rollup_and_prune_query_logs(cutoff) {
                        if deleted > 0 {
                            tracing::info!(
                                "DNS Retention: Rolled up and pruned {} detailed raw query logs older than {} days",
                                deleted,
                                retention_days
                            );
                            let _ =
                                storage_maint.set_meta("dns_log_last_pruned_at", &now.to_rfc3339());
                        }
                    }
                }

                maint_interval.tick().await;
            }
        });

        // 8. Periodic Process Monitoring Task (every 5 seconds)
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
                                            check_and_enforce_app_quota(
                                                &ev.process_name,
                                                &storage,
                                                &telemetry_pipeline,
                                            );
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

                            // Continuously enforce daily quota on running applications
                            if let Ok(rules) = storage.list_local_rules() {
                                if rules.iter().any(|r| r.rule_type == "application" && r.enabled) {
                                    if let Ok(procs) = famshield_windows::ProcessMonitor::enumerate_processes() {
                                        for p in procs {
                                            check_and_enforce_app_quota(
                                                &p.name,
                                                &storage,
                                                &telemetry_pipeline,
                                            );
                                        }
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
                let _ = famshield_windows::NrptManager::remove_nrpt_rule();
            }
        }
    }
}

/// Evaluates and enforces daily screen time limits for a given application.
/// If the application has exceeded its allotted quota, all its running instances are terminated
/// and a security alert is recorded in the telemetry pipeline.
/// Returns true if quota was exceeded and enforcement occurred.
pub fn check_and_enforce_app_quota(
    proc_name: &str,
    storage: &famshield_storage::StorageEngine,
    telemetry_pipeline: &Arc<TelemetryPipeline>,
) -> bool {
    let mut enforced = false;
    if let Ok(rules) = storage.list_local_rules() {
        let proc_lower = proc_name.to_lowercase();
        let proc_base = proc_lower.trim_end_matches(".exe");
        for r in rules {
            if r.rule_type == "application" && r.enabled {
                let rule_lower = r.rule_id.to_lowercase();
                let rule_base = rule_lower.trim_end_matches(".exe");
                if proc_lower == rule_lower || proc_base == rule_base {
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&r.payload) {
                        let max_minutes = parsed
                            .get("max_minutes_per_day")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0);
                        if max_minutes > 0 {
                            if let Ok(usage_ms) = storage.get_app_usage_today_ms(proc_name) {
                                let used_minutes = (usage_ms / 60_000) as u64;
                                if used_minutes >= max_minutes {
                                    tracing::warn!(
                                        "⛔ Application '{}' exceeded daily quota (used {} / limit {} mins)! Terminating process...",
                                        proc_name,
                                        used_minutes,
                                        max_minutes
                                    );
                                    let killed = famshield_windows::ProcessMonitor::terminate_processes_by_name(
                                        proc_name,
                                    )
                                    .unwrap_or(0);
                                    tracing::info!(
                                        "Enforcement: Terminated {} instances of '{}'",
                                        killed,
                                        proc_name
                                    );

                                    let now = chrono::Utc::now();
                                    let alert_payload = serde_json::json!({
                                        "process_name": proc_name,
                                        "used_minutes": used_minutes,
                                        "max_minutes": max_minutes,
                                        "action": "TERMINATE_QUOTA_EXCEEDED",
                                        "killed_count": killed,
                                        "timestamp": now.to_rfc3339(),
                                    });
                                    let _ = telemetry_pipeline.queue_event(&TelemetryEvent::new(
                                        format!("quota-term-{}", now.timestamp_micros()),
                                        "security_alert",
                                        now,
                                        alert_payload,
                                    ));

                                    let _ = famshield_windows::SessionManager::lock_workstation();
                                    enforced = true;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    enforced
}

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};

use crate::client::{CloudSyncClient, HeartbeatRequest};
use crate::telemetry::TelemetryPipeline;
use famshield_dns::blocklist::BlocklistStore;
use famshield_storage::StorageEngine;

/// Known domain blacklist seeds corresponding to category flags
const ADULT_DOMAINS: &[&str] = &[
    "pornhub.com",
    "xvideos.com",
    "xnxx.com",
    "redtube.com",
    "onlyfans.com",
    "chaturbate.com",
    "stripchat.com",
    "livejasmin.com",
    "spankbang.com",
];

const GAMBLING_DOMAINS: &[&str] = &[
    "bet365.com",
    "1xbet.com",
    "w88.com",
    "fun88.com",
    "m88.com",
    "fb88.com",
    "dafabet.com",
    "pokerstars.com",
    "stake.com",
];

const VIOLENCE_DOMAINS: &[&str] = &[
    "bestgore.com",
    "crazyshit.com",
    "liveleak.com",
    "kaotic.com",
    "theync.com",
];

const SOCIAL_DOMAINS: &[&str] = &[
    "tiktok.com",
    "facebook.com",
    "instagram.com",
    "twitter.com",
    "x.com",
    "reddit.com",
    "snapchat.com",
];

const GAMING_DOMAINS: &[&str] = &[
    "roblox.com",
    "steampowered.com",
    "steamcommunity.com",
    "riotgames.com",
    "epicgames.com",
    "garena.vn",
    "minecraft.net",
];

/// Background background runner managing periodic Telemetry ingestion, Heartbeat, and Policy synchronization.
pub struct TelemetrySyncWorker {
    pipeline: Arc<TelemetryPipeline>,
    client: Arc<CloudSyncClient>,
    storage: StorageEngine,
    dns_blocklist: Option<Arc<BlocklistStore>>,
    start_time: Instant,
}

impl TelemetrySyncWorker {
    pub fn new(
        pipeline: Arc<TelemetryPipeline>,
        client: Arc<CloudSyncClient>,
        storage: StorageEngine,
        dns_blocklist: Option<Arc<BlocklistStore>>,
    ) -> Self {
        Self {
            pipeline,
            client,
            storage,
            dns_blocklist,
            start_time: Instant::now(),
        }
    }

    /// Run the background synchronization loop until shutdown signal is received.
    pub async fn run(&self, mut shutdown_rx: broadcast::Receiver<()>) {
        info!(
            "Starting Telemetry & Cloud Sync Worker (target: {})...",
            self.client.base_url()
        );

        let mut telemetry_interval = tokio::time::interval(Duration::from_secs(15));
        let mut heartbeat_interval = tokio::time::interval(Duration::from_secs(60));

        loop {
            tokio::select! {
                _ = telemetry_interval.tick() => {
                    self.sync_outbox_telemetry().await;
                }
                _ = heartbeat_interval.tick() => {
                    self.sync_heartbeat_and_policy().await;
                }
                _ = shutdown_rx.recv() => {
                    info!("TelemetrySyncWorker shutting down upon signal");
                    break;
                }
            }
        }
    }

    /// Check if endpoint is enrolled in Online Managed Mode.
    fn is_enrolled(&self) -> (bool, String, String) {
        let dev_id = self
            .storage
            .get_meta(crate::enrollment::META_ENROLLED_DEVICE_ID)
            .ok()
            .flatten()
            .unwrap_or_default();
        let tenant_id = self
            .storage
            .get_meta(crate::enrollment::META_ENROLLED_TENANT_ID)
            .ok()
            .flatten()
            .unwrap_or_default();

        let is_enrolled = !dev_id.trim().is_empty();
        (is_enrolled, dev_id, tenant_id)
    }

    /// Dynamically resolve CloudSyncClient with latest server_url from storage
    fn get_sync_client(&self) -> Arc<CloudSyncClient> {
        if let Ok(Some(url)) = self.storage.get_meta("server_url") {
            let trimmed = url.trim().to_string();
            if !trimmed.is_empty() && trimmed != self.client.base_url() {
                return Arc::new(CloudSyncClient::new(&trimmed));
            }
        }
        self.client.clone()
    }

    /// Push pending outbox events to Cloud Server
    async fn sync_outbox_telemetry(&self) {
        let (enrolled, _dev_id, _tenant_id) = self.is_enrolled();
        if !enrolled {
            debug!("Device running in StandaloneFree mode, skipping cloud telemetry sync");
            return;
        }

        let client = self.get_sync_client();

        // Dynamically scale batch size based on pending outbox depth
        let pending = self.pipeline.pending_count().unwrap_or(0);
        let batch_size = if pending > 100 { 100 } else { 50 };

        // Process up to MAX_BATCHES_PER_CYCLE per interval to prevent task starvation
        const MAX_BATCHES_PER_CYCLE: usize = 10;
        let mut batches_sent = 0;

        while batches_sent < MAX_BATCHES_PER_CYCLE {
            match self.pipeline.prepare_next_batch(batch_size) {
                Ok(Some(envelope)) => {
                    let event_count = envelope.event_count;
                    let batch_id = envelope.batch_id.clone();

                    match client.send_batch_events(&envelope).await {
                        Ok(resp) => {
                            if let Err(e) = self.pipeline.acknowledge_batch(&envelope) {
                                warn!("Failed to acknowledge sent telemetry batch: {}", e);
                                break; // Halt to avoid resending the same batch in an infinite loop
                            } else {
                                info!(
                                    "Successfully synchronized telemetry batch {} ({} events accepted)",
                                    batch_id,
                                    resp.accepted.len()
                                );
                                batches_sent += 1;
                            }
                        }
                        Err(e) => {
                            warn!(
                                "Failed to upload telemetry batch {} ({} events): {}. Will retry later.",
                                batch_id, event_count, e
                            );
                            let _ = self.pipeline.report_batch_failure(&envelope, 30);
                            break; // Back off on network error
                        }
                    }
                }
                Ok(None) => break,
                Err(e) => {
                    warn!("Failed to prepare telemetry batch from outbox: {}", e);
                    break;
                }
            }
        }
    }

    /// Send Heartbeat, report health metrics, and pull policy updates if version increased
    async fn sync_heartbeat_and_policy(&self) {
        let (enrolled, dev_id, tenant_id) = self.is_enrolled();
        let client = self.get_sync_client();
        if !enrolled {
            // Attempt auto-enrollment if enrollment_code was pre-configured via bootstrap
            if let Ok(Some(code)) = self.storage.get_meta("enrollment_code") {
                let code = code.trim();
                if code.len() == 6 {
                    let enroll_manager =
                        crate::EnrollmentManager::new(self.storage.clone(), client.base_url());
                    let custom_name = self.storage.get_meta("device_name").ok().flatten();
                    let device_name =
                        famshield_windows::get_disambiguated_device_name(custom_name.as_deref());
                    let _ = self.storage.set_meta("device_name", &device_name);
                    if let Some(guid) = famshield_windows::get_machine_guid() {
                        let _ = self.storage.set_meta("machine_guid", &guid);
                    }
                    let req = crate::EnrollmentRequest {
                        enrollment_code: code.to_string(),
                        device_name,
                        os_version: "Windows".to_string(),
                    };
                    match enroll_manager.enroll(&req).await {
                        Ok(resp) => {
                            info!("Auto-enrollment succeeded: Device ID {}", resp.device_id);
                            let _ = self.storage.set_meta("enrollment_code", "");
                        }
                        Err(e) => {
                            warn!("Auto-enrollment attempt failed: {}", e);
                        }
                    }
                }
            }
            return;
        }

        let uptime_seconds = self.start_time.elapsed().as_secs();
        let agent_version = env!("CARGO_PKG_VERSION").to_string();
        let outbox_depth = self.pipeline.pending_count().unwrap_or(0);

        let current_policy_ver = self
            .storage
            .get_meta("policy_version")
            .ok()
            .flatten()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(1);

        let optimal_upstream = self.storage.get_meta("dns_upstream").ok().flatten();
        let hb_req = HeartbeatRequest {
            device_id: dev_id.clone(),
            agent_version: agent_version.clone(),
            policy_version: current_policy_ver,
            health_status: Some("healthy".to_string()),
            status: Some("healthy".to_string()),
            uptime_seconds,
            wfp_active: true,
            dns_active: true,
            outbox_depth,
            dns_upstream_optimal: optimal_upstream,
            dns_avg_rtt_ms: None,
        };

        match client.send_heartbeat(&hb_req).await {
            Ok(resp) => {
                debug!(
                    "Heartbeat succeeded. Server time: {}, Desired policy: {}",
                    resp.server_time, resp.desired_policy_version
                );

                // Check if cloud server has newer policy version
                if resp.desired_policy_version > current_policy_ver && !tenant_id.is_empty() {
                    info!(
                        "Cloud policy update detected (local v{} -> cloud v{}). Pulling...",
                        current_policy_ver, resp.desired_policy_version
                    );
                    self.update_policy_from_cloud(&tenant_id, resp.desired_policy_version)
                        .await;
                }

                // Check if auto-update release is available
                if let Some(rel) = resp.available_release {
                    if famshield_core::is_newer_version(&rel.version, &agent_version) {
                        info!(
                            "New FamShield agent update available: v{} (current: v{}). Channel: {}",
                            rel.version, agent_version, rel.channel
                        );
                        if let Ok(json) = serde_json::to_string(&rel) {
                            let _ = self.storage.set_meta("available_release_json", &json);
                            let _ = self.storage.set_meta("available_release_ver", &rel.version);
                        }
                    } else {
                        // Purge any stale/older cached release metadata
                        let _ = self.storage.delete_meta("available_release_json");
                        let _ = self.storage.delete_meta("available_release_ver");
                    }
                } else {
                    // No release available from server, purge stale cached release
                    let _ = self.storage.delete_meta("available_release_json");
                    let _ = self.storage.delete_meta("available_release_ver");
                }

                // Dynamically sync updated server URL or version check URL from Server
                if let Some(ref s_url) = resp.server_url {
                    let s_url = s_url.trim();
                    if !s_url.is_empty()
                        && self
                            .storage
                            .get_meta("server_url")
                            .ok()
                            .flatten()
                            .as_deref()
                            != Some(s_url)
                    {
                        info!("Server instructed update of server_url to: {}", s_url);
                        let _ = self.storage.set_meta("server_url", s_url);
                    }
                }
                if let Some(ref v_url) = resp.version_check_url {
                    let v_url = v_url.trim();
                    if !v_url.is_empty()
                        && self
                            .storage
                            .get_meta("version_check_url")
                            .ok()
                            .flatten()
                            .as_deref()
                            != Some(v_url)
                    {
                        info!(
                            "Server instructed update of version_check_url to: {}",
                            v_url
                        );
                        let _ = self.storage.set_meta("version_check_url", v_url);
                    }
                }
            }
            Err(e) => {
                warn!("Heartbeat sync error: {}", e);
            }
        }
    }

    /// Pull policy and update local DNS blocklist store dynamically
    async fn update_policy_from_cloud(&self, tenant_id: &str, new_version: u32) {
        match self.client.fetch_web_filter_policy(tenant_id).await {
            Ok(pol) => {
                info!(
                    "Received WebFilterPolicy v{}: adult={}, gambling={}, violence={}, social={}, gaming={}",
                    pol.version,
                    pol.block_adult,
                    pol.block_gambling,
                    pol.block_violence,
                    pol.block_social,
                    pol.block_gaming
                );

                // Merge category seeds into active domain list
                let mut block_domains: Vec<String> = pol.custom_blocked_domains.clone();

                // Merge local custom rules from endpoint storage to preserve parent rules
                if let Ok(local_rules) = self.storage.list_local_rules() {
                    for r in local_rules {
                        if r.rule_type == "domain" && r.enabled {
                            block_domains.push(r.rule_id);
                        }
                    }
                }

                if pol.block_adult {
                    block_domains.extend(ADULT_DOMAINS.iter().map(|d| d.to_string()));
                }
                if pol.block_gambling {
                    block_domains.extend(GAMBLING_DOMAINS.iter().map(|d| d.to_string()));
                }
                if pol.block_violence {
                    block_domains.extend(VIOLENCE_DOMAINS.iter().map(|d| d.to_string()));
                }
                if pol.block_social {
                    block_domains.extend(SOCIAL_DOMAINS.iter().map(|d| d.to_string()));
                }
                if pol.block_gaming {
                    block_domains.extend(GAMING_DOMAINS.iter().map(|d| d.to_string()));
                }

                block_domains.sort();
                block_domains.dedup();

                // If allowlist has domains, exclude them
                if !pol.custom_allowed_domains.is_empty() {
                    let allows: std::collections::HashSet<String> =
                        pol.custom_allowed_domains.iter().cloned().collect();
                    block_domains.retain(|d| !allows.contains(d));
                }

                // Update runtime DNS proxy blocklist in memory
                if let Some(blocklist) = &self.dns_blocklist {
                    blocklist.replace_blocklist(block_domains.clone());
                    blocklist.set_safesearch_enabled(pol.safesearch_enabled);
                    blocklist.set_youtube_restricted(pol.youtube_restricted);
                    info!(
                        "Dynamic DNS blocklist refreshed with {} domains (safesearch={}, youtube_restricted={}, silent_mode={})",
                        block_domains.len(),
                        pol.safesearch_enabled,
                        pol.youtube_restricted,
                        pol.silent_mode
                    );
                }

                // Persist silent_mode in storage
                let _ = self.storage.set_meta(
                    "silent_mode",
                    if pol.silent_mode { "true" } else { "false" },
                );

                if pol.monitor_only_mode {
                    let _ = self.storage.set_meta("monitor_mode", "true");
                    if let Some(blocklist) = &self.dns_blocklist {
                        blocklist.set_monitor_mode(true);
                    }
                }

                // Cache full cloud policy object in storage for local UI and DNS sync
                if let Ok(policy_str) = serde_json::to_string(&pol) {
                    let _ = self.storage.set_meta("cached_cloud_policy", &policy_str);
                }

                // Persist new policy version in storage
                let _ = self
                    .storage
                    .set_meta("policy_version", &new_version.to_string());
                let _ = self
                    .storage
                    .set_meta("last_policy_sync", &chrono::Utc::now().to_rfc3339());
            }
            Err(e) => {
                error!("Failed to fetch policy from cloud: {}", e);
            }
        }
    }
}

use futures_util::StreamExt;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};

use crate::nats_dispatcher::{NatsDispatcher, RemoteCommand};
use famshield_storage::StorageEngine;

/// NatsConnectionWorker maintains a persistent, duplex NATS connection
/// (optimized for Cloudflare Tunnel over Secure WebSocket `wss://` or raw TCP `nats://`).
pub struct NatsConnectionWorker {
    device_id: String,
    nats_url: String,
    dispatcher: Arc<NatsDispatcher>,
    storage: StorageEngine,
    telemetry_pipeline: Option<Arc<crate::telemetry::TelemetryPipeline>>,
}

impl NatsConnectionWorker {
    pub fn new(
        device_id: impl Into<String>,
        nats_url: impl Into<String>,
        dispatcher: Arc<NatsDispatcher>,
        storage: StorageEngine,
    ) -> Self {
        Self {
            device_id: device_id.into(),
            nats_url: nats_url.into(),
            dispatcher,
            storage,
            telemetry_pipeline: None,
        }
    }

    /// Attach a TelemetryPipeline to enable WorkQueue-based event draining over NATS
    pub fn with_telemetry_pipeline(mut self, pipeline: Arc<crate::telemetry::TelemetryPipeline>) -> Self {
        self.telemetry_pipeline = Some(pipeline);
        self
    }

    /// Check if endpoint is enrolled in Online Managed Mode.
    fn is_enrolled(&self) -> bool {
        let dev_id = self
            .storage
            .get_meta(crate::enrollment::META_ENROLLED_DEVICE_ID)
            .ok()
            .flatten()
            .unwrap_or_default();
        !dev_id.trim().is_empty()
    }

    /// Resolve NATS URL dynamically from storage (handles runtime enrollment / URL changes)
    fn get_nats_url(&self) -> String {
        if let Ok(Some(url)) = self.storage.get_meta("nats_url") {
            let trimmed = url.trim();
            if !trimmed.is_empty() {
                return famshield_core::normalize_nats_url(trimmed);
            }
        }
        if let Ok(Some(server_url)) = self.storage.get_meta("server_url") {
            let trimmed = server_url.trim();
            if !trimmed.is_empty() {
                return famshield_core::normalize_nats_url(trimmed);
            }
        }
        famshield_core::normalize_nats_url(&self.nats_url)
    }

    /// Run the persistent NATS subscription worker loop until shutdown signal.
    pub async fn run(&self, mut shutdown_rx: broadcast::Receiver<()>) {
        info!(
            "Starting NATS Real-Time Control Plane Worker (URL: {})...",
            self.nats_url
        );

        let mut backoff_secs = 2u64;

        loop {
            // Check if device is enrolled
            if !self.is_enrolled() {
                debug!("Device running in StandaloneFree mode, NATS worker sleeping...");
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        info!("NATS worker shutting down (StandaloneFree mode)");
                        return;
                    }
                    _ = tokio::time::sleep(Duration::from_secs(30)) => {
                        continue;
                    }
                }
            }

            let current_nats_url = self.get_nats_url();

            // Attempt to connect to NATS Broker
            info!("Connecting to NATS broker at {}...", current_nats_url);
            let connect_opts = async_nats::ConnectOptions::new()
                .connection_timeout(Duration::from_secs(10))
                .ping_interval(Duration::from_secs(20)); // Keeps Cloudflare Tunnel WebSocket from timing out

            let client = match connect_opts.connect(&current_nats_url).await {
                Ok(c) => {
                    info!(
                        "Successfully connected to NATS broker at {}",
                        current_nats_url
                    );
                    let _ = self.storage.set_meta("nats_status", "connected");
                    let _ = self
                        .storage
                        .set_meta("nats_last_connected_at", &chrono::Utc::now().to_rfc3339());
                    let _ = self.storage.delete_meta("nats_last_error");
                    backoff_secs = 2; // Reset backoff on success
                    c
                }
                Err(e) => {
                    warn!(
                        "Failed to connect to NATS broker ({}): {}. Retrying in {}s...",
                        current_nats_url, e, backoff_secs
                    );
                    let _ = self.storage.set_meta("nats_status", "disconnected");
                    let _ = self.storage.set_meta("nats_last_error", &e.to_string());
                    tokio::select! {
                        _ = shutdown_rx.recv() => {
                            info!("NATS worker shutting down upon signal during backoff");
                            return;
                        }
                        _ = tokio::time::sleep(Duration::from_secs(backoff_secs)) => {
                            backoff_secs = (backoff_secs * 2).min(30);
                            continue;
                        }
                    }
                }
            };

            // Subscribe to device's dedicated command subject: devices.{device_id}.commands
            let command_sub = self.dispatcher.command_subject();
            let mut subscriber = match client.subscribe(command_sub.clone()).await {
                Ok(s) => {
                    info!("Subscribed to real-time command topic: {}", command_sub);
                    s
                }
                Err(e) => {
                    error!("Failed to subscribe to {}: {}", command_sub, e);
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }
            };

            // Main message listening loop
            let mut connection_dropped = false;
            let mut rollup_interval = tokio::time::interval(Duration::from_secs(60));
            let mut telemetry_interval = tokio::time::interval(Duration::from_secs(15));
            let mut heartbeat_interval = tokio::time::interval(Duration::from_secs(60));

            while !connection_dropped {
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        info!("NATS worker received shutdown signal, disconnecting cleanly.");
                        return;
                    }
                    _ = telemetry_interval.tick() => {
                        self.publish_pending_telemetry(&client).await;
                    }
                    _ = heartbeat_interval.tick() => {
                        self.publish_realtime_heartbeat(&client).await;
                    }
                    _ = rollup_interval.tick() => {
                        self.publish_periodic_rollup(&client).await;
                    }
                    msg_opt = subscriber.next() => {
                        match msg_opt {
                            Some(msg) => {
                                self.handle_incoming_message(&msg, &client).await;
                            }
                            None => {
                                warn!("NATS subscriber stream ended (connection dropped). Reconnecting...");
                                let _ = self.storage.set_meta("nats_status", "disconnected");
                                connection_dropped = true;
                            }
                        }
                    }
                }
            }

            // Brief pause before reconnecting
            tokio::time::sleep(Duration::from_secs(backoff_secs)).await;
        }
    }

    /// Process a received command message, verify HMAC, dispatch, and report ACK
    async fn handle_incoming_message(
        &self,
        msg: &async_nats::Message,
        client: &async_nats::Client,
    ) {
        let payload_str = match std::str::from_utf8(&msg.payload) {
            Ok(s) => s,
            Err(e) => {
                warn!("Received non-UTF8 payload on NATS: {}", e);
                return;
            }
        };

        debug!("Received command payload: {}", payload_str);
        let cmd: RemoteCommand = match serde_json::from_str(payload_str) {
            Ok(c) => c,
            Err(e) => {
                warn!("Failed to deserialize RemoteCommand JSON: {}", e);
                return;
            }
        };

        let cmd_id = cmd.command_id.clone();
        let cmd_type = cmd.command_type.clone();

        match self.dispatcher.dispatch_command(&cmd) {
            Ok(()) => {
                info!(
                    "Command '{}' (ID: {}) executed successfully in real-time.",
                    cmd_type, cmd_id
                );

                // Publish ACK back to server: devices.{device_id}.command_results
                let ack_subject = format!("devices.{}.command_results", self.device_id);
                let ack_payload = serde_json::json!({
                    "command_id": cmd_id,
                    "device_id": self.device_id,
                    "state": "succeeded",
                    "completed_at": chrono::Utc::now().to_rfc3339(),
                    "code": "OK"
                });

                if let Ok(ack_bytes) = serde_json::to_vec(&ack_payload) {
                    if let Err(e) = client.publish(ack_subject, ack_bytes.into()).await {
                        warn!("Failed to publish command execution ACK: {}", e);
                    }
                }
            }
            Err(err) => {
                warn!(
                    "Rejected or failed to execute command '{}' (ID: {}): {}",
                    cmd_type, cmd_id, err
                );

                // Publish Failure ACK
                let ack_subject = format!("devices.{}.command_results", self.device_id);
                let ack_payload = serde_json::json!({
                    "command_id": cmd_id,
                    "device_id": self.device_id,
                    "state": "failed",
                    "completed_at": chrono::Utc::now().to_rfc3339(),
                    "code": format!("{}", err)
                });

                if let Ok(ack_bytes) = serde_json::to_vec(&ack_payload) {
                    let _ = client.publish(ack_subject, ack_bytes.into()).await;
                }
            }
        }
    }

    /// Periodically aggregate domain queries and application screen time, pushing a rollup summary to NATS
    async fn publish_periodic_rollup(&self, client: &async_nats::Client) {
        let domain_rollups = match self.storage.get_domain_rollup_summaries(50) {
            Ok(list) => list,
            Err(e) => {
                debug!("No domain rollups found or error reading storage: {}", e);
                Vec::new()
            }
        };

        let app_usage = match self.storage.get_app_usage_grouped(None, None) {
            Ok(list) => list,
            Err(e) => {
                debug!("No app usage found or error reading storage: {}", e);
                Vec::new()
            }
        };

        if domain_rollups.is_empty() && app_usage.is_empty() {
            return;
        }

        let apps: Vec<crate::nats_dispatcher::AppUsageRollupItem> = app_usage
            .into_iter()
            .map(|a| crate::nats_dispatcher::AppUsageRollupItem {
                app_name: a.app_name,
                total_duration_minutes: (a.total_duration_ms / 60_000).max(1) as u64,
                sessions: a.session_count,
                last_active: a.last_seen_utc,
            })
            .collect();

        match self.dispatcher.format_rollup_summary(domain_rollups, apps) {
            Ok(json_payload) => {
                let subject = format!("devices.{}.events.rollup", self.device_id);
                if let Err(e) = client
                    .publish(subject.clone(), json_payload.clone().into())
                    .await
                {
                    warn!(
                        "Failed to publish rollup to NATS subject {}: {}",
                        subject, e
                    );
                } else {
                    info!(
                        "Pushed web & app rollup summary to NATS ({} bytes)",
                        json_payload.len()
                    );
                }
                // Also publish to JetStream stream subject for backend event ingestion
                let _ = client
                    .publish("famshield.events.web_rollup", json_payload.into())
                    .await;
            }
            Err(e) => {
                warn!("Failed to format rollup summary: {}", e);
            }
        }
    }

    /// Drain pending Outbox events and publish to NATS JetStream WorkQueue
    async fn publish_pending_telemetry(&self, client: &async_nats::Client) {
        let Some(pipeline) = &self.telemetry_pipeline else {
            return;
        };

        const MAX_BATCHES_PER_CYCLE: usize = 5;
        let mut batches_sent = 0;

        while batches_sent < MAX_BATCHES_PER_CYCLE {
            match pipeline.prepare_next_batch(50) {
                Ok(Some(envelope)) => {
                    let subject = format!("devices.{}.events.telemetry", self.device_id);
                    let event_ids: Vec<String> =
                        envelope.events.iter().map(|e| e.event_id.clone()).collect();
                    let batch_id = envelope.batch_id.clone();
                    let count = envelope.event_count;

                    match envelope.to_json() {
                        Ok(json_str) => {
                            match client.publish(subject.clone(), json_str.into()).await {
                                Ok(_) => {
                                    if let Err(e) =
                                        pipeline.acknowledge_events(&envelope, &event_ids)
                                    {
                                        warn!("Failed to acknowledge NATS telemetry events: {}", e);
                                        break;
                                    }
                                    info!(
                                        "Pushed telemetry batch {} ({} events) to NATS WorkQueue successfully",
                                        batch_id, count
                                    );
                                    batches_sent += 1;
                                }
                                Err(e) => {
                                    warn!(
                                        "Failed to publish telemetry to NATS subject {}: {}",
                                        subject, e
                                    );
                                    let _ = pipeline.report_batch_failure(&envelope, 30);
                                    break;
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Failed to serialize telemetry envelope: {}", e);
                            break;
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

    /// Send lightweight real-time heartbeat over NATS
    async fn publish_realtime_heartbeat(&self, client: &async_nats::Client) {
        let subject = format!("devices.{}.heartbeat", self.device_id);
        let current_policy_ver = self
            .storage
            .get_meta("policy_version")
            .ok()
            .flatten()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(1);
        let outbox_depth = self
            .telemetry_pipeline
            .as_ref()
            .and_then(|p| p.pending_count().ok())
            .unwrap_or(0);

        let hb_req = serde_json::json!({
            "device_id": self.device_id,
            "agent_version": env!("CARGO_PKG_VERSION"),
            "policy_version": current_policy_ver,
            "status": "healthy",
            "wfp_active": true,
            "dns_active": true,
            "outbox_depth": outbox_depth,
        });

        if let Ok(data) = serde_json::to_vec(&hb_req) {
            let _ = client.publish(subject, data.into()).await;
        }
    }
}

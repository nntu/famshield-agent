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
        }
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
                    backoff_secs = 2; // Reset backoff on success
                    c
                }
                Err(e) => {
                    warn!(
                        "Failed to connect to NATS broker ({}): {}. Retrying in {}s...",
                        current_nats_url, e, backoff_secs
                    );
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
            while !connection_dropped {
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        info!("NATS worker received shutdown signal, disconnecting cleanly.");
                        return;
                    }
                    msg_opt = subscriber.next() => {
                        match msg_opt {
                            Some(msg) => {
                                self.handle_incoming_message(&msg, &client).await;
                            }
                            None => {
                                warn!("NATS subscriber stream ended (connection dropped). Reconnecting...");
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
}

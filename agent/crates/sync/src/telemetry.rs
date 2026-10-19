use chrono::{DateTime, Duration, Utc};
use famshield_storage::StorageEngine;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::error::SyncError;

pub const CURRENT_TELEMETRY_VERSION: &str = "1.0";
pub const DEFAULT_BATCH_SIZE: usize = 50;
pub const DEFAULT_RETRY_BACKOFF_SECS: i64 = 30;

/// Standardized Telemetry Event Type
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TelemetryEventType {
    Heartbeat,
    DnsQuery,
    ProcessEvent,
    ErrorLog,
    VisualEvidence,
    SecurityAlert,
    #[serde(untagged)]
    Custom(String),
}

impl TelemetryEventType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Heartbeat => "heartbeat",
            Self::DnsQuery => "dns_query",
            Self::ProcessEvent => "process_event",
            Self::ErrorLog => "error_log",
            Self::VisualEvidence => "visual_evidence",
            Self::SecurityAlert => "security_alert",
            Self::Custom(s) => s.as_str(),
        }
    }
}

/// Standardized Individual Telemetry Event
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TelemetryEvent {
    pub event_id: String,
    pub event_type: String,
    pub occurred_at: DateTime<Utc>,
    pub payload: serde_json::Value,
}

impl TelemetryEvent {
    pub fn new(
        event_id: impl Into<String>,
        event_type: impl Into<String>,
        occurred_at: DateTime<Utc>,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            event_id: event_id.into(),
            event_type: event_type.into(),
            occurred_at,
            payload,
        }
    }
}

/// Unified Telemetry Envelope sent from Client to Server
/// Standardizes transport across Cloudflare Tunnel HTTP POST and NATS JetStream
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TelemetryEnvelope {
    pub version: String,
    pub device_id: String,
    pub batch_id: String,
    pub sent_at: DateTime<Utc>,
    pub event_count: usize,
    pub events: Vec<TelemetryEvent>,
}

impl TelemetryEnvelope {
    pub fn new(device_id: impl Into<String>, events: Vec<TelemetryEvent>) -> Self {
        let count = events.len();
        let now = Utc::now();
        let dev = device_id.into();
        let batch_id = format!("batch-{}-{}", dev, now.timestamp_millis());
        Self {
            version: CURRENT_TELEMETRY_VERSION.to_string(),
            device_id: dev,
            batch_id,
            sent_at: now,
            event_count: count,
            events,
        }
    }

    pub fn to_json(&self) -> Result<String, SyncError> {
        serde_json::to_string(self).map_err(SyncError::Serialization)
    }

    pub fn to_json_pretty(&self) -> Result<String, SyncError> {
        serde_json::to_string_pretty(self).map_err(SyncError::Serialization)
    }
}

/// Strongly-typed Payload Models for Common Events
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct HeartbeatPayload {
    pub uptime_seconds: u64,
    pub agent_version: String,
    pub wfp_active: bool,
    pub dns_active: bool,
    pub pending_outbox_count: usize,
    pub status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DnsQueryPayload {
    pub domain: String,
    pub qtype: String,
    pub action: String, // "allow" | "block"
    pub reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SecurityAlertPayload {
    pub alert_type: String, // "pin_lockout", "wfp_tamper", "driver_bypass"
    pub severity: String,   // "WARN", "HIGH", "CRITICAL"
    pub description: String,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// TelemetryPipeline orchestrates offline persistence (SQLite outbox)
/// and batch packaging for transport.
pub struct TelemetryPipeline {
    device_id: String,
    storage: StorageEngine,
}

impl TelemetryPipeline {
    pub fn new(device_id: impl Into<String>, storage: StorageEngine) -> Self {
        Self {
            device_id: device_id.into(),
            storage,
        }
    }

    pub fn device_id(&self) -> &str {
        &self.device_id
    }

    /// Queue a telemetry event to local SQLite outbox (Instant, non-blocking, offline-safe).
    pub fn queue_event(&self, event: &TelemetryEvent) -> Result<(), SyncError> {
        let payload_str = serde_json::to_string(&event.payload)?;
        self.storage.enqueue_outbox(
            &event.event_id,
            &event.event_type,
            event.occurred_at,
            &payload_str,
        )?;
        Ok(())
    }

    /// Helper to queue a heartbeat event.
    pub fn queue_heartbeat(
        &self,
        uptime_seconds: u64,
        agent_version: &str,
        wfp_active: bool,
        dns_active: bool,
    ) -> Result<String, SyncError> {
        let count = self.storage.count_outbox().unwrap_or(0);
        let payload = HeartbeatPayload {
            uptime_seconds,
            agent_version: agent_version.to_string(),
            wfp_active,
            dns_active,
            pending_outbox_count: count,
            status: "HEALTHY".to_string(),
        };

        let now = Utc::now();
        let event_id = format!("hb-{}-{}", self.device_id, now.timestamp_millis());
        let event = TelemetryEvent::new(
            &event_id,
            TelemetryEventType::Heartbeat.as_str(),
            now,
            serde_json::to_value(&payload)?,
        );

        self.queue_event(&event)?;
        Ok(event_id)
    }

    /// Helper to queue a DNS query log event.
    pub fn queue_dns_query(
        &self,
        domain: &str,
        qtype: &str,
        action: &str,
        reason: &str,
        rule_id: Option<&str>,
        occurred_at: DateTime<Utc>,
    ) -> Result<String, SyncError> {
        let payload = DnsQueryPayload {
            domain: domain.to_string(),
            qtype: qtype.to_string(),
            action: action.to_string(),
            reason: reason.to_string(),
            rule_id: rule_id.map(|s| s.to_string()),
        };

        let event_id = format!(
            "dns-{}-{}",
            self.device_id,
            occurred_at
                .timestamp_nanos_opt()
                .unwrap_or_else(|| occurred_at.timestamp_millis() * 1_000_000)
        );
        let event = TelemetryEvent::new(
            &event_id,
            TelemetryEventType::DnsQuery.as_str(),
            occurred_at,
            serde_json::to_value(&payload)?,
        );

        self.queue_event(&event)?;
        Ok(event_id)
    }

    /// Helper to queue a security alert (e.g. PIN lockout).
    pub fn queue_security_alert(
        &self,
        alert_type: &str,
        severity: &str,
        description: &str,
        metadata: serde_json::Value,
    ) -> Result<String, SyncError> {
        let payload = SecurityAlertPayload {
            alert_type: alert_type.to_string(),
            severity: severity.to_string(),
            description: description.to_string(),
            metadata,
        };

        let now = Utc::now();
        let event_id = format!("sec-{}-{}", self.device_id, now.timestamp_millis());
        let event = TelemetryEvent::new(
            &event_id,
            TelemetryEventType::SecurityAlert.as_str(),
            now,
            serde_json::to_value(&payload)?,
        );

        self.queue_event(&event)?;
        Ok(event_id)
    }

    /// Assemble the next pending telemetry batch into a unified envelope ready for transport.
    pub fn prepare_next_batch(
        &self,
        max_batch_size: usize,
    ) -> Result<Option<TelemetryEnvelope>, SyncError> {
        let records = self.storage.peek_outbox(max_batch_size)?;

        if records.is_empty() {
            return Ok(None);
        }

        let mut events = Vec::with_capacity(records.len());
        for rec in records {
            let payload: serde_json::Value =
                serde_json::from_str(&rec.payload).unwrap_or(serde_json::Value::Null);
            events.push(TelemetryEvent {
                event_id: rec.event_id,
                event_type: rec.event_type,
                occurred_at: rec.occurred_at,
                payload,
            });
        }

        let envelope = TelemetryEnvelope::new(&self.device_id, events);
        Ok(Some(envelope))
    }

    /// Confirm successful receipt by server; purge the sent events from outbox.
    pub fn acknowledge_batch(&self, envelope: &TelemetryEnvelope) -> Result<(), SyncError> {
        let event_ids: Vec<String> = envelope.events.iter().map(|e| e.event_id.clone()).collect();
        self.storage.dequeue_outbox(&event_ids)?;
        info!(
            "Acknowledged telemetry batch {} ({} events)",
            envelope.batch_id, envelope.event_count
        );
        Ok(())
    }

    /// Report transmission failure; schedule retry with exponential backoff.
    pub fn report_batch_failure(
        &self,
        envelope: &TelemetryEnvelope,
        backoff_seconds: i64,
    ) -> Result<(), SyncError> {
        let event_ids: Vec<String> = envelope.events.iter().map(|e| e.event_id.clone()).collect();
        let next_retry = Utc::now() + Duration::seconds(backoff_seconds);
        self.storage.fail_outbox_events(&event_ids, next_retry)?;
        warn!(
            "Telemetry batch {} transmission failed; retrying in {}s",
            envelope.batch_id, backoff_seconds
        );
        Ok(())
    }

    /// Total pending events count in outbox.
    pub fn pending_count(&self) -> Result<usize, SyncError> {
        Ok(self.storage.count_outbox()?)
    }
}

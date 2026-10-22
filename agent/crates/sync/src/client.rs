use crate::enrollment::{EnrollmentRequest, EnrollmentResponse};
use crate::error::SyncError;
use crate::telemetry::TelemetryEnvelope;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, info};

/// Heartbeat payload sent to `POST /v1/device/heartbeat`
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HeartbeatRequest {
    pub device_id: String,
    pub agent_version: String,
    pub policy_version: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    pub uptime_seconds: u64,
    pub wfp_active: bool,
    pub dns_active: bool,
    pub outbox_depth: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dns_upstream_optimal: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dns_avg_rtt_ms: Option<u64>,
}

/// Metadata for available OTA software release from server
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AvailableReleaseDto {
    pub manifest_version: u32,
    pub version: String,
    pub channel: String,
    pub min_supported_version: String,
    pub service_sha256: String,
    pub service_binary_url: String,
    pub signature_hex: String,
}

/// Heartbeat response received from `POST /v1/device/heartbeat`
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HeartbeatResponse {
    pub server_time: String,
    #[serde(default)]
    pub desired_policy_version: u32,
    #[serde(default)]
    pub command_wait_seconds: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_release: Option<AvailableReleaseDto>,
    #[serde(default)]
    pub server_url: Option<String>,
    #[serde(default)]
    pub version_check_url: Option<String>,
}

/// Batch events response from `POST /v1/device/events:batch`
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BatchEventsResponse {
    pub batch_id: String,
    #[serde(default)]
    pub accepted: Vec<String>,
    #[serde(default)]
    pub duplicate: Vec<String>,
}

/// Web filter policy definition fetched from server
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WebFilterPolicyDto {
    pub tenant_id: String,
    #[serde(default)]
    pub version: u32,
    #[serde(default)]
    pub block_adult: bool,
    #[serde(default)]
    pub block_gambling: bool,
    #[serde(default)]
    pub block_violence: bool,
    #[serde(default)]
    pub block_social: bool,
    #[serde(default)]
    pub block_gaming: bool,
    #[serde(default)]
    pub safesearch_enabled: bool,
    #[serde(default)]
    pub youtube_restricted: bool,
    #[serde(default)]
    pub custom_blocked_domains: Vec<String>,
    #[serde(default)]
    pub custom_allowed_domains: Vec<String>,
}

/// High-performance HTTP client for communicating with FamShield Cloud Server.
#[derive(Clone)]
pub struct CloudSyncClient {
    client: reqwest::Client,
    base_url: String,
}

impl CloudSyncClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        let mut url = base_url.into().trim().to_string();
        if url.ends_with('/') {
            url.pop();
        }
        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(15))
            .tcp_keepalive(Some(Duration::from_secs(30)))
            .tcp_nodelay(true)
            .pool_idle_timeout(Some(Duration::from_secs(90)))
            .pool_max_idle_per_host(8)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            client,
            base_url: url,
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Negotiate cloud enrollment with 6-digit code: `POST /v1/device/enroll`
    pub async fn enroll(&self, req: &EnrollmentRequest) -> Result<EnrollmentResponse, SyncError> {
        let url = format!("{}/v1/device/enroll", self.base_url);
        info!("Sending enrollment request to {}", url);

        let res =
            self.client.post(&url).json(req).send().await.map_err(|e| {
                SyncError::Network(format!("Failed to reach enrollment server: {}", e))
            })?;

        if !res.status().is_success() {
            let status = res.status();
            let err_body = res.text().await.unwrap_or_default();
            return Err(SyncError::Enrollment(format!(
                "Enrollment rejected (status {}): {}",
                status, err_body
            )));
        }

        let resp: EnrollmentResponse = res.json().await.map_err(|e| {
            SyncError::Enrollment(format!("Invalid enrollment response JSON: {}", e))
        })?;

        info!("Device enrolled successfully. ID: {}", resp.device_id);
        Ok(resp)
    }

    /// Report health, telemetry counters, and check for policy updates: `POST /v1/device/heartbeat`
    pub async fn send_heartbeat(
        &self,
        req: &HeartbeatRequest,
    ) -> Result<HeartbeatResponse, SyncError> {
        let url = format!("{}/v1/device/heartbeat", self.base_url);
        debug!("Sending heartbeat to {}", url);

        let res = self
            .client
            .post(&url)
            .json(req)
            .send()
            .await
            .map_err(|e| SyncError::Network(format!("Heartbeat network error: {}", e)))?;

        if !res.status().is_success() {
            let status = res.status();
            let err_body = res.text().await.unwrap_or_default();
            return Err(SyncError::Network(format!(
                "Heartbeat failed (status {}): {}",
                status, err_body
            )));
        }

        let resp: HeartbeatResponse = res
            .json()
            .await
            .map_err(|e| SyncError::Network(format!("Invalid heartbeat response JSON: {}", e)))?;

        Ok(resp)
    }

    /// Send a batch of queued telemetry events from SQLite outbox: `POST /v1/device/events:batch`
    pub async fn send_batch_events(
        &self,
        envelope: &TelemetryEnvelope,
    ) -> Result<BatchEventsResponse, SyncError> {
        let url = format!("{}/v1/device/events:batch", self.base_url);
        debug!(
            "Uploading telemetry batch {} ({} events) to {}",
            envelope.batch_id, envelope.event_count, url
        );

        let res = self
            .client
            .post(&url)
            .json(envelope)
            .send()
            .await
            .map_err(|e| SyncError::Network(format!("Telemetry batch upload failed: {}", e)))?;

        if !res.status().is_success() {
            let status = res.status();
            let err_body = res.text().await.unwrap_or_default();
            return Err(SyncError::Network(format!(
                "Batch rejected by server (status {}): {}",
                status, err_body
            )));
        }

        let resp: BatchEventsResponse = res
            .json()
            .await
            .map_err(|e| SyncError::Network(format!("Invalid batch response JSON: {}", e)))?;

        Ok(resp)
    }

    /// Fetch latest web filter policy: `GET /v1/filter/policy?tenant_id=...`
    pub async fn fetch_web_filter_policy(
        &self,
        tenant_id: &str,
    ) -> Result<WebFilterPolicyDto, SyncError> {
        let url = format!("{}/v1/filter/policy?tenant_id={}", self.base_url, tenant_id);
        debug!("Fetching web filter policy from {}", url);

        let res = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| SyncError::Network(format!("Failed to fetch policy: {}", e)))?;

        if !res.status().is_success() {
            let status = res.status();
            return Err(SyncError::Network(format!(
                "Failed to fetch policy (status {})",
                status
            )));
        }

        let policy: WebFilterPolicyDto = res
            .json()
            .await
            .map_err(|e| SyncError::Network(format!("Invalid policy JSON: {}", e)))?;

        Ok(policy)
    }

    /// Update web filter policy on cloud server: `POST /v1/filter/policy?tenant_id=...`
    pub async fn update_web_filter_policy(
        &self,
        tenant_id: &str,
        policy: &WebFilterPolicyDto,
    ) -> Result<WebFilterPolicyDto, SyncError> {
        let url = format!("{}/v1/filter/policy?tenant_id={}", self.base_url, tenant_id);
        debug!("Pushing updated web filter policy to {}", url);

        let res = self
            .client
            .post(&url)
            .json(policy)
            .send()
            .await
            .map_err(|e| SyncError::Network(format!("Failed to update policy: {}", e)))?;

        if !res.status().is_success() {
            let status = res.status();
            let err_body = res.text().await.unwrap_or_default();
            return Err(SyncError::Network(format!(
                "Failed to update policy (status {}): {}",
                status, err_body
            )));
        }

        let resp: serde_json::Value = res.json().await.map_err(|e| {
            SyncError::Network(format!("Invalid policy update response JSON: {}", e))
        })?;

        if let Some(pol_val) = resp.get("policy") {
            if let Ok(pol) = serde_json::from_value::<WebFilterPolicyDto>(pol_val.clone()) {
                return Ok(pol);
            }
        }

        Ok(policy.clone())
    }
}

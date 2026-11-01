use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]

pub struct LoginRequest {
    pub pin: String,
}

#[derive(Serialize)]

pub struct LoginResponse {
    pub success: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_totp: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub temp_auth_token: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Deserialize)]

pub struct TotpLoginRequest {
    pub temp_auth_token: String,

    pub code: String,
}

#[derive(Deserialize)]

pub struct ConfirmTotpRequest {
    pub code: String,
}

#[derive(Deserialize)]

pub struct DisableTotpRequest {
    pub pin: String,

    pub code: Option<String>,
}

#[derive(Serialize)]

pub struct VerifyResponse {
    pub authenticated: bool,
}

#[derive(Serialize)]

pub struct StatusResponse {
    pub protection_active: bool,

    pub monitor_mode: bool,

    pub silent_mode: bool,

    pub dns_block_action: String,

    pub session_helper_running: bool,

    pub session_helper_status: String,

    pub dns_proxy_addr: String,

    pub wfp_status: String,

    pub operating_mode: String,

    pub device_id: String,

    pub active_foreground_app: String,

    pub daily_quota_minutes: u32,

    pub used_today_minutes: u32,

    pub blocked_queries_today: usize,

    pub total_queries_today: usize,

    pub rules_count: usize,

    pub evidence_count: usize,

    pub app_version: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_info: Option<famshield_windows::SystemInfo>,
}

#[derive(Deserialize, Default)]

pub struct ProtectionModeRequest {
    #[serde(default)]
    pub monitor_mode: Option<bool>,

    #[serde(default)]
    pub silent_mode: Option<bool>,

    #[serde(default)]
    pub dns_block_action: Option<String>,
}

#[derive(Serialize)]

pub struct ProtectionModeResponse {
    pub success: bool,

    pub monitor_mode: bool,

    pub silent_mode: bool,

    pub dns_block_action: String,
}

#[derive(Deserialize)]

pub struct SilentModeRequest {
    pub silent_mode: bool,
}

#[derive(Serialize)]

pub struct SilentModeResponse {
    pub success: bool,

    pub silent_mode: bool,
}

#[derive(Serialize)]

pub struct EvidenceItemResponse {
    pub evidence_id: String,

    pub event_id: String,

    pub occurred_at: DateTime<Utc>,

    pub trigger: String,

    pub size_bytes: usize,

    pub expires_at: DateTime<Utc>,

    pub thumbnail_url: String,
}

#[derive(Deserialize)]

pub struct ChangePinRequest {
    pub current_pin: Option<String>,

    pub recovery_key: Option<String>,

    pub new_pin: String,
}

#[derive(Deserialize)]

pub struct CloudEnrollRequest {
    pub enrollment_code: String,
    pub device_name: Option<String>,
}

#[derive(Deserialize)]

pub struct TestConnectionRequest {
    pub server_url: String,
}

#[derive(Serialize)]

pub struct SetupStatusResponse {
    pub is_configured: bool,

    pub operating_mode: String,

    pub device_id: String,

    pub app_version: String,
}

#[derive(Deserialize)]

pub struct InitializeSetupRequest {
    pub pin: String,

    pub profile: Option<String>,

    pub blocked_categories: Option<Vec<String>>,

    pub operating_mode: Option<String>,

    pub enrollment_code: Option<String>,

    pub server_url: Option<String>,
}

#[derive(Serialize)]

pub struct InitializeSetupResponse {
    pub success: bool,

    pub token: String,

    pub recovery_key: String,

    pub rules_created: usize,
}

#[derive(Deserialize, Default)]

pub struct QueryParams {
    pub limit: Option<usize>,

    pub token: Option<String>,

    pub domain: Option<String>,

    pub action: Option<String>,

    pub start_time: Option<String>,

    pub end_time: Option<String>,

    pub period: Option<String>,

    pub exclude_system: Option<bool>,
}

use chrono::{DateTime, Utc};
use famshield_storage::{LocalRuleRecord, StorageError};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use thiserror::Error;
use crate::auth::AuthError;

pub enum LocalAdminError {
    #[error("Authentication failed: {0}")]
    Auth(#[from] AuthError),

    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    #[error("Policy schema validation failed: {0}")]
    PolicyInvalid(String),

    #[error("Unauthorized access")]
    Unauthorized,

    #[error("Backup error: {0}")]
    Backup(String),
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CreateRuleRequest {
    pub rule_id: String,
    pub rule_type: String, // "application" | "domain"
    pub payload_json: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub operating_mode: String,
    pub server_url: String,
    pub version_check_url: Option<String>,
    pub tenant_id: String,
    pub device_id: String,
    pub device_name: String,
    pub enrollment_code: Option<String>,
    pub sync_interval_secs: u32,
    pub tls_verify: bool,
    pub connection_status: String,
    pub last_sync_time: Option<String>,
    pub policy_version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemSettings {
    pub server_url: String,

    pub version_check_url: String,

    pub nats_url: String,

    pub sync_interval_secs: u32,

    pub heartbeat_interval_secs: u32,

    pub dns_upstream: String,

    pub dns_block_action: String,

    pub dns_cache_ttl_secs: u32,

    pub dns_timeout_ms: u64,

    pub silent_mode: bool,

    pub monitor_mode: bool,

    pub screenshot_blur_mode: String,

    pub dns_log_retention_days: u32,

    pub dns_auto_prune_enabled: bool,

    pub keyboard_monitoring_enabled: bool,

    pub max_pin_attempts: u32,

    pub pin_lockout_duration_mins: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]

pub struct UpdateSystemSettingsRequest {
    pub server_url: Option<String>,

    pub version_check_url: Option<String>,

    pub nats_url: Option<String>,

    pub sync_interval_secs: Option<u32>,

    pub heartbeat_interval_secs: Option<u32>,

    pub dns_upstream: Option<String>,

    pub dns_block_action: Option<String>,

    pub dns_cache_ttl_secs: Option<u32>,

    pub dns_timeout_ms: Option<u64>,

    pub silent_mode: Option<bool>,

    pub monitor_mode: Option<bool>,

    pub screenshot_blur_mode: Option<String>,

    pub dns_log_retention_days: Option<u32>,

    pub dns_auto_prune_enabled: Option<bool>,

    pub keyboard_monitoring_enabled: Option<bool>,

    pub max_pin_attempts: Option<u32>,

    pub pin_lockout_duration_mins: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateServerConfigRequest {
    pub operating_mode: Option<String>,
    pub server_url: Option<String>,
    pub version_check_url: Option<String>,
    pub tenant_id: Option<String>,
    pub device_name: Option<String>,
    pub enrollment_code: Option<String>,
    pub sync_interval_secs: Option<u32>,
    pub tls_verify: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConnectionResult {
    pub success: bool,
    pub latency_ms: u64,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupSettings {
    pub operating_mode: String,
    pub server_url: String,
    pub tenant_id: String,
    pub device_name: String,
    pub sync_interval_secs: u32,
    pub tls_verify: bool,
    pub monitor_mode: bool,
    #[serde(default)]
    pub silent_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupPayload {
    pub format_version: String,
    pub created_at: String,
    pub app_version: String,
    pub device_id: String,
    pub settings: BackupSettings,
    pub rules: Vec<LocalRuleRecord>,
    pub checksum: String,
}

impl BackupPayload {
    pub fn compute_checksum(
        format_version: &str,
        created_at: &str,
        device_id: &str,
        settings_json: &str,
        rules_json: &str,
    ) -> String {
        let mut hasher = Sha256::new();
        hasher.update(format_version.as_bytes());
        hasher.update(created_at.as_bytes());
        hasher.update(device_id.as_bytes());
        hasher.update(settings_json.as_bytes());
        hasher.update(rules_json.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn verify_checksum(&self) -> bool {
        let Ok(settings_json) = serde_json::to_string(&self.settings) else {
            return false;
        };
        let Ok(rules_json) = serde_json::to_string(&self.rules) else {
            return false;
        };
        let expected = Self::compute_checksum(
            &self.format_version,
            &self.created_at,
            &self.device_id,
            &settings_json,
            &rules_json,
        );
        expected == self.checksum
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreResult {
    pub success: bool,
    pub rules_restored: usize,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppActivityItem {
    pub id: String,
    pub app_name: String,
    pub app_title: String,
    pub category: String,
    #[serde(default)]
    pub category_id: String,
    #[serde(default)]
    pub is_system: bool,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub duration_minutes: u32,
    pub duration_formatted: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebActivityItem {
    pub id: usize,
    pub domain: String,
    pub category: String,
    pub action: String,
    pub reason: String,
    pub occurred_at: DateTime<Utc>,
    pub source_app: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedTimelineItem {
    pub id: String,
    pub item_type: String,
    pub title: String,
    pub subtitle: String,
    pub category: String,
    pub action: String,
    pub timestamp: DateTime<Utc>,
    pub duration_minutes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeywordRuleDto {
    pub id: String,
    pub pattern: String,
    pub category: String,
    pub category_name: String,
    pub severity: String,
    pub enabled: bool,
    pub is_custom: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeywordViolationDto {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub app_name: String,
    pub keyword_matched: String,
    pub category: String,
    pub severity: String,
    pub snippet_masked: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddKeywordRequest {
    pub pattern: String,
    pub category: Option<String>,
    pub severity: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToggleKeywordRequest {
    pub id: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeywordConfigResponse {
    pub keyboard_monitoring_enabled: bool,
    pub total_rules_count: usize,
    pub violations_today_count: u64,
    pub rules: Vec<KeywordRuleDto>,
    pub recent_violations: Vec<KeywordViolationDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppUsageSummary {
    pub app_name: String,
    pub app_title: String,
    pub category: String,
    #[serde(default)]
    pub category_id: String,
    #[serde(default)]
    pub is_system: bool,
    pub total_duration_minutes: u32,
    pub total_duration_formatted: String,
    pub percentage: f32,
    pub session_count: usize,
    pub first_seen_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    pub is_active_now: bool,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppCategoryMetadata {
    pub id: String,
    pub name_vi: String,
    pub name_en: String,
    pub icon: String,
    pub is_system: bool,
    pub description: String,
    pub app_count: usize,
    pub total_duration_minutes: u32,
    pub percentage: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeAggregatedReport {
    pub period: String,
    pub start_date: String,
    pub end_date: String,
    pub total_active_minutes: u32,
    pub total_system_minutes: u32,
    pub total_screen_time_formatted: String,
    pub user_apps_count: usize,
    pub system_apps_count: usize,
    pub category_breakdown: Vec<AppCategoryUsageBreakdown>,
    pub hourly_distribution: Vec<HourlyUsagePoint>,
    pub top_user_apps: Vec<AppUsageSummary>,
    pub all_apps: Vec<AppUsageSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppCategoryUsageBreakdown {
    pub category_id: String,
    pub category_name: String,
    pub total_minutes: u32,
    pub percentage: f32,
    pub app_count: usize,
    pub icon: String,
    pub is_system: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HourlyUsagePoint {
    pub hour: u8,
    pub label: String,
    pub user_minutes: u32,
    pub system_minutes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivitySummaryResponse {
    pub date: String,
    pub total_screen_time_minutes: u32,
    pub total_screen_time_formatted: String,
    pub active_apps_count: usize,
    pub total_web_visits_count: usize,
    pub blocked_events_count: usize,
    pub monitored_events_count: usize,
    pub keyword_violations_count: usize,
    pub app_activities: Vec<AppActivityItem>,
    pub grouped_apps: Vec<AppUsageSummary>,
    pub web_activities: Vec<WebActivityItem>,
    pub keyword_violations: Vec<KeywordViolationDto>,
    pub unified_timeline: Vec<UnifiedTimelineItem>,
}


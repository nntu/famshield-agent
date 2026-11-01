use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryLogRecord {
    pub domain: String,
    pub qtype: String,
    pub action: String,
    pub reason: String,
    pub rule_id: Option<String>,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainQuerySummary {
    pub domain: String,
    pub total_queries: usize,
    pub allowed_count: usize,
    pub blocked_count: usize,
    pub monitored_count: usize,
    pub error_count: usize,
    pub last_seen_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainRollupItem {
    pub domain: String,
    pub app_name: String,
    pub total_hits: usize,
    pub allowed_hits: usize,
    pub blocked_hits: usize,
    pub last_seen_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HourlyFrequency {
    pub hour: u32,
    pub total_count: u64,
    pub allowed_count: u64,
    pub blocked_count: u64,
}

#[derive(Clone, Debug, Default)]
pub struct QueryLogFilter {
    pub domain: Option<String>,
    pub action: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub limit: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocalRuleRecord {
    pub rule_id: String,
    pub rule_type: String,
    pub payload: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default = "default_local_source")]
    pub source: String,
}

fn default_local_source() -> String {
    "local".to_string()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VisualEvidenceRecord {
    pub evidence_id: String,
    pub event_id: String,
    pub occurred_at: DateTime<Utc>,
    pub trigger: String,
    pub image_payload: Vec<u8>,
    pub size_bytes: usize,
    pub expires_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ErrorLogRecord {
    pub error_id: String,
    pub module: String,
    pub severity: String, // 'WARN', 'ERROR', 'FATAL'
    pub message: String,
    pub stack_trace: Option<String>,
    pub occurrence_count: u32,
    pub first_seen_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    pub synced: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutboxRecord {
    pub event_id: String,
    pub event_type: String,
    pub occurred_at: DateTime<Utc>,
    pub payload: String,
    pub attempts: u32,
    pub next_attempt_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UsageSegmentRecord {
    pub segment_id: String,
    pub child_id: String,
    pub rule_id: String,
    pub start_utc: DateTime<Utc>,
    pub duration_ms: i64,
    pub state: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppUsageGroupRecord {
    pub app_name: String,
    pub total_duration_ms: i64,
    pub session_count: usize,
    pub first_seen_utc: DateTime<Utc>,
    pub last_seen_utc: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HourlyAppUsageRecord {
    pub hour: u8,
    pub app_name: String,
    pub duration_minutes: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeywordViolationRecord {
    pub id: String,
    pub timestamp_utc: DateTime<Utc>,
    pub app_name: String,
    pub keyword_matched: String,
    pub category: String,
    pub severity: String,
    pub snippet_masked: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomKeywordRecord {
    pub id: String,
    pub pattern: String,
    pub category: String,
    pub severity: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppCategoryEntity {
    pub id: String,
    pub name_vi: String,
    pub name_en: String,
    pub icon: String,
    pub is_system: bool,
    pub description_vi: Option<String>,
    pub description_en: Option<String>,
    pub sort_order: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppCategoryRuleEntity {
    pub rule_id: String,
    pub category_id: String,
    pub pattern: String,
    pub match_type: String, // 'contains', 'exact', 'starts_with', 'ends_with'
    pub app_title: String,
    pub is_system: bool,
    pub priority: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResetReportStats {
    pub dns_cleared: usize,
    pub apps_cleared: usize,
    pub evidence_cleared: usize,
    pub keywords_cleared: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParentTotpConfigRecord {
    pub enabled: bool,
    pub encrypted_secret: String,
    pub created_at: DateTime<Utc>,
    pub last_verified_at: Option<DateTime<Utc>>,
}

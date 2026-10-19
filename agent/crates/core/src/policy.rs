use crate::domain::ApplicationCategory;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Policy {
    pub schema_version: u32,
    pub policy_id: String,
    pub version: u64,
    pub timezone: String,
    #[serde(default = "default_idle_threshold")]
    pub idle_threshold_seconds: u32,
    pub application: ApplicationConfig,
    pub web: WebConfig,
    #[serde(default)]
    pub visual_verification: Option<VisualVerificationConfig>,
    pub telemetry: TelemetryConfig,
}

fn default_idle_threshold() -> u32 {
    300
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApplicationConfig {
    pub enabled: bool,
    pub default_action: RuleAction,
    #[serde(default)]
    pub enforcement_tier: EnforcementTier,
    #[serde(default)]
    pub rules: Vec<ApplicationRule>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EnforcementTier {
    #[default]
    UserModeOnly,
    HybridWdac,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleAction {
    Allow,
    Block,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApplicationRule {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub action: RuleAction,
    pub identity: ApplicationIdentity,
    pub schedule: Schedule,
    #[serde(default)]
    pub warn_before_minutes: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ApplicationIdentity {
    #[serde(default)]
    pub executable_names: Vec<String>,
    #[serde(default)]
    pub canonical_paths: Vec<String>,
    #[serde(default)]
    pub signer_thumbprints: Vec<String>,
    #[serde(default)]
    pub sha256: Vec<String>,
    #[serde(default)]
    pub application_families: Vec<String>,
    #[serde(default)]
    pub categories: Vec<ApplicationCategory>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebConfig {
    pub enabled: bool,
    pub mode: WebMode,
    pub dns_failure_mode: DnsFailureMode,
    pub block_external_dns: bool,
    pub block_doh: bool,
    #[serde(default = "default_true")]
    pub wfp_enforcement: bool,
    #[serde(default)]
    pub safe_search: Option<SafeSearchConfig>,
    #[serde(default)]
    pub rules: Vec<DomainRule>,
}

fn default_true() -> bool {
    true
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WebMode {
    AllowByDefault,
    DenyByDefault,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DnsFailureMode {
    AllowWithAlert,
    Block,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SafeSearchConfig {
    #[serde(default = "default_true")]
    pub search_engines: bool,
    #[serde(default)]
    pub youtube_restricted: YouTubeRestrictedMode,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum YouTubeRestrictedMode {
    #[default]
    Off,
    Moderate,
    Strict,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainRule {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub action: RuleAction,
    pub domains: Vec<String>,
    pub schedule: Schedule,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Schedule {
    #[serde(default)]
    pub daily_quota_minutes: Option<u32>,
    #[serde(default)]
    pub windows: Vec<TimeWindow>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeWindow {
    pub days: Vec<DayOfWeek>,
    pub start: String, // "HH:MM"
    pub end: String,   // "HH:MM"
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "snake_case")]
pub enum DayOfWeek {
    Mon,
    Tue,
    Wed,
    Thu,
    Fri,
    Sat,
    Sun,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VisualVerificationConfig {
    pub enabled: bool,
    pub triggers: Vec<VisualTrigger>,
    #[serde(default = "default_blur")]
    pub blur_level: BlurLevel,
    #[serde(default = "default_daily_captures")]
    pub max_daily_captures: u32,
    #[serde(default = "default_retention_hours")]
    pub retention_hours: u32,
}

fn default_blur() -> BlurLevel {
    BlurLevel::Medium
}

fn default_daily_captures() -> u32 {
    50
}

fn default_retention_hours() -> u32 {
    48
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VisualTrigger {
    AppBlocked,
    DomainBlocked,
    TimeRequested,
    SuspiciousAnomaly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BlurLevel {
    Low,
    #[default]
    Medium,
    High,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TelemetryConfig {
    pub application_usage: bool,
    pub domain_observation: bool,
    pub endpoint_retention_days: u32,
}

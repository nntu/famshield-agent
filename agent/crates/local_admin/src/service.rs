use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::auth::{AuthError, MasterPinAuthenticator};
use famshield_storage::{LocalRuleRecord, StorageEngine, StorageError};

#[derive(Debug, Error)]
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
pub struct UpdateServerConfigRequest {
    pub operating_mode: Option<String>,
    pub server_url: Option<String>,
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

pub fn classify_domain(domain: &str) -> &'static str {
    let lower = domain.trim().to_ascii_lowercase();
    if lower.contains("youtube")
        || lower.contains("tiktok")
        || lower.contains("netflix")
        || lower.contains("twitch")
        || lower.contains("spotify")
        || lower.contains("bilibili")
    {
        "Giải trí & Video (Entertainment)"
    } else if lower.contains("facebook")
        || lower.contains("instagram")
        || lower.contains("discord")
        || lower.contains("twitter")
        || lower.contains("zalo")
        || lower.contains("threads")
        || lower.contains("reddit")
    {
        "Mạng xã hội (Social Media)"
    } else if lower.contains("roblox")
        || lower.contains("steam")
        || lower.contains("epicgames")
        || lower.contains("riot")
        || lower.contains("minecraft")
        || lower.contains("game")
    {
        "Trò chơi (Gaming)"
    } else if lower.contains("google")
        || lower.contains("bing")
        || lower.contains("duckduckgo")
        || lower.contains("coccoc")
    {
        "Công cụ Tìm kiếm (Search Engine)"
    } else if lower.contains("wikipedia")
        || lower.contains("coursera")
        || lower.contains("duolingo")
        || lower.contains("khanacademy")
        || lower.contains("edu")
    {
        "Học tập & Giáo dục (Education)"
    } else if lower.contains("github")
        || lower.contains("stackoverflow")
        || lower.contains("rust-lang")
        || lower.contains("developer")
    {
        "Lập trình & Kỹ thuật (Development)"
    } else {
        "Trang web Chung (Web / Other)"
    }
}

pub fn classify_app(app_name: &str) -> (&'static str, &'static str) {
    let lower = app_name.trim().to_ascii_lowercase();
    if lower.contains("roblox") {
        ("Roblox Player", "Trò chơi (Gaming)")
    } else if lower.contains("minecraft") || lower.contains("javaw") {
        ("Minecraft", "Trò chơi (Gaming)")
    } else if lower.contains("genshin") {
        ("Genshin Impact", "Trò chơi (Gaming)")
    } else if lower.contains("league") || lower.contains("riot") || lower.contains("valorant") {
        ("Riot Games (LoL/Valorant)", "Trò chơi (Gaming)")
    } else if lower.contains("steam") || lower.contains("epicgames") {
        ("Game Launcher (Steam/Epic)", "Trò chơi (Gaming)")
    } else if lower.contains("csgo") || lower.contains("cs2") || lower.contains("dota") {
        ("Valve Game (CS/Dota)", "Trò chơi (Gaming)")
    } else if lower.contains("ldplayer") || lower.contains("nox") || lower.contains("bluestacks") {
        ("Giả lập Android Game", "Trò chơi (Gaming)")
    } else if lower.contains("chrome") {
        ("Google Chrome", "Trình duyệt (Browser)")
    } else if lower.contains("msedge") {
        ("Microsoft Edge", "Trình duyệt (Browser)")
    } else if lower.contains("coccoc") {
        ("Cốc Cốc Browser", "Trình duyệt (Browser)")
    } else if lower.contains("firefox") || lower.contains("brave") || lower.contains("opera") {
        ("Trình duyệt Web", "Trình duyệt (Browser)")
    } else if lower.contains("discord") {
        ("Discord", "Mạng xã hội (Social Media)")
    } else if lower.contains("zalo") {
        ("Zalo", "Liên lạc (Communication)")
    } else if lower.contains("telegram")
        || lower.contains("messenger")
        || lower.contains("whatsapp")
    {
        ("Nhắn tin (Chat)", "Liên lạc (Communication)")
    } else if lower.contains("spotify") || lower.contains("vlc") || lower.contains("mediaplayer") {
        ("Nghe nhạc & Video", "Giải trí (Entertainment)")
    } else if lower.contains("code")
        || lower.contains("devenv")
        || lower.contains("idea64")
        || lower.contains("pycharm")
    {
        ("Công cụ Lập trình (IDE)", "Lập trình (Development)")
    } else if lower.contains("zoom") || lower.contains("teams") {
        ("Hội họp Trực tuyến", "Học tập & Làm việc (Education/Work)")
    } else if lower.contains("winword")
        || lower.contains("word")
        || lower.contains("excel")
        || lower.contains("powerpnt")
    {
        ("Microsoft Office", "Học tập & Làm việc (Education/Work)")
    } else if lower.contains("acrobat")
        || lower.contains("acrord32")
        || lower.contains("foxit")
        || lower.contains("notion")
    {
        ("Tài liệu & Ghi chú", "Học tập & Làm việc (Education/Work)")
    } else {
        ("Ứng dụng Windows", "Tiện ích & Hệ thống (Utilities)")
    }
}

pub struct LocalAdminService {
    storage: StorageEngine,
}

impl LocalAdminService {
    pub fn new(storage: StorageEngine) -> Self {
        Self { storage }
    }

    /// Check whether the initial Master PIN has been configured.
    pub fn is_configured(&self) -> bool {
        MasterPinAuthenticator::is_pin_configured(&self.storage)
    }

    /// Authenticate guardian using Master PIN against local storage.
    pub fn authenticate_storage(&self, pin: &str) -> Result<bool, LocalAdminError> {
        let valid = MasterPinAuthenticator::verify_storage_pin(&self.storage, pin)?;
        Ok(valid)
    }

    /// Set initial or new Master PIN, returning Emergency Recovery Key.
    pub fn set_master_pin(&self, pin: &str) -> Result<String, LocalAdminError> {
        let key = MasterPinAuthenticator::set_master_pin(&self.storage, pin)?;
        Ok(key)
    }

    /// Reset Master PIN using Emergency Recovery Key.
    pub fn reset_pin_with_recovery_key(
        &self,
        recovery_key: &str,
        new_pin: &str,
    ) -> Result<(), LocalAdminError> {
        MasterPinAuthenticator::reset_pin_with_recovery_key(&self.storage, recovery_key, new_pin)?;
        Ok(())
    }

    /// Reset Master PIN with elevated Administrator authorization.
    pub fn reset_pin_elevated(&self, new_pin: &str) -> Result<(), LocalAdminError> {
        MasterPinAuthenticator::reset_pin_elevated(&self.storage, new_pin)?;
        Ok(())
    }

    /// Authenticate guardian using Master PIN against local storage with lockout protection.
    pub fn authenticate(&self, pin: &str) -> Result<bool, LocalAdminError> {
        self.authenticate_storage(pin)
    }

    /// Save a local rule in SQLite after verifying structure.
    pub fn create_rule(&self, req: CreateRuleRequest) -> Result<(), LocalAdminError> {
        let now = Utc::now();
        let record = LocalRuleRecord {
            rule_id: req.rule_id,
            rule_type: req.rule_type,
            payload: req.payload_json,
            enabled: req.enabled,
            created_at: now,
            updated_at: now,
        };

        self.storage.upsert_local_rule(&record)?;
        Ok(())
    }

    /// Save multiple local rules in a batch.
    pub fn create_rules_bulk(
        &self,
        requests: Vec<CreateRuleRequest>,
    ) -> Result<usize, LocalAdminError> {
        let now = Utc::now();
        let mut count = 0;
        for req in requests {
            let record = LocalRuleRecord {
                rule_id: req.rule_id,
                rule_type: req.rule_type,
                payload: req.payload_json,
                enabled: req.enabled,
                created_at: now,
                updated_at: now,
            };
            self.storage.upsert_local_rule(&record)?;
            count += 1;
        }
        Ok(count)
    }

    /// List all local rules currently stored in the standalone database.
    pub fn list_rules(&self) -> Result<Vec<LocalRuleRecord>, LocalAdminError> {
        let rules = self.storage.list_local_rules()?;
        Ok(rules)
    }

    /// Delete a local rule by ID.
    pub fn delete_rule(&self, rule_id: &str) -> Result<bool, LocalAdminError> {
        let res = self.storage.delete_local_rule(rule_id)?;
        Ok(res)
    }

    /// Query recent DNS logs for local reporting.
    pub fn list_query_logs(
        &self,
        limit: usize,
    ) -> Result<Vec<famshield_storage::QueryLogRecord>, LocalAdminError> {
        let logs = self.storage.list_recent_query_logs(limit)?;
        Ok(logs)
    }

    /// Query aggregated DNS summaries grouped by domain.
    pub fn get_domain_query_summaries(
        &self,
        limit: usize,
    ) -> Result<Vec<famshield_storage::DomainQuerySummary>, LocalAdminError> {
        let summaries = self.storage.get_domain_query_summaries(limit)?;
        Ok(summaries)
    }

    /// Query aggregated DNS summaries grouped by domain with optional time range filter.
    pub fn get_domain_query_summaries_filtered(
        &self,
        start_time: Option<chrono::DateTime<chrono::Utc>>,
        end_time: Option<chrono::DateTime<chrono::Utc>>,
        limit: usize,
    ) -> Result<Vec<famshield_storage::DomainQuerySummary>, LocalAdminError> {
        let summaries = self
            .storage
            .get_domain_query_summaries_filtered(start_time, end_time, limit)?;
        Ok(summaries)
    }

    /// Query 24-hour access frequency distribution.
    pub fn get_hourly_frequency(
        &self,
        domain: Option<&str>,
        start_time: Option<chrono::DateTime<chrono::Utc>>,
        end_time: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Vec<famshield_storage::HourlyFrequency>, LocalAdminError> {
        let hourly = self
            .storage
            .get_hourly_frequency(domain, start_time, end_time)?;
        Ok(hourly)
    }

    /// Query DNS logs with multi-criteria filtering (domain, action, time range, limit).
    pub fn list_query_logs_with_filter(
        &self,
        filter: &famshield_storage::QueryLogFilter,
    ) -> Result<Vec<famshield_storage::QueryLogRecord>, LocalAdminError> {
        let logs = self.storage.list_query_logs_filtered(filter)?;
        Ok(logs)
    }

    /// Query DNS logs with optional domain filtering.
    pub fn list_query_logs_filtered(
        &self,
        domain: Option<&str>,
        limit: usize,
    ) -> Result<Vec<famshield_storage::QueryLogRecord>, LocalAdminError> {
        if let Some(d) = domain {
            let trimmed = d.trim();
            if !trimmed.is_empty() {
                let logs = self.storage.list_query_logs_for_domain(trimmed, limit)?;
                return Ok(logs);
            }
        }
        self.list_query_logs(limit)
    }

    /// Query visual evidence gallery for local guardian review.
    pub fn list_visual_evidence(
        &self,
        limit: usize,
    ) -> Result<Vec<famshield_storage::VisualEvidenceRecord>, LocalAdminError> {
        let list = self.storage.list_visual_evidence(limit)?;
        Ok(list)
    }

    /// Retrieve the JPEG image payload of a specific visual evidence record.
    pub fn get_visual_evidence_image(
        &self,
        evidence_id: &str,
    ) -> Result<Option<Vec<u8>>, LocalAdminError> {
        let record = self.storage.get_visual_evidence_by_id(evidence_id)?;
        Ok(record.map(|r| r.image_payload))
    }

    /// Access reference to underlying storage engine.
    pub fn storage(&self) -> &StorageEngine {
        &self.storage
    }

    /// Retrieve centralized server connection configuration and live telemetry
    pub fn get_server_config(&self) -> Result<ServerConfig, LocalAdminError> {
        let enrolled_id = self
            .storage
            .get_meta("enrolled_device_id")?
            .unwrap_or_default();
        let is_enrolled = !enrolled_id.trim().is_empty();

        let server_url = self
            .storage
            .get_meta("server_url")?
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "https://api.famshield.cloud".to_string());

        let tenant_id = self
            .storage
            .get_meta("enrolled_tenant_id")?
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "tenant-default".to_string());

        let device_name = self
            .storage
            .get_meta("device_name")?
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "DESKTOP-FAMSHIELD".to_string());

        let enrollment_code = self
            .storage
            .get_meta("enrollment_code")?
            .filter(|s| !s.is_empty());

        let sync_interval_secs = self
            .storage
            .get_meta("sync_interval_secs")?
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(60);

        let tls_verify = self
            .storage
            .get_meta("tls_verify")?
            .map(|s| s != "false")
            .unwrap_or(true);

        let last_sync_time = self
            .storage
            .get_meta("last_sync_time")?
            .filter(|s| !s.is_empty());

        let policy_version = self
            .storage
            .get_meta("policy_version")?
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(1);

        let (operating_mode, connection_status) = if is_enrolled {
            ("OnlineManaged".to_string(), "Connected".to_string())
        } else {
            ("StandaloneFree".to_string(), "Disconnected".to_string())
        };

        Ok(ServerConfig {
            operating_mode,
            server_url,
            tenant_id,
            device_id: if is_enrolled {
                enrolled_id
            } else {
                "standalone-local-001".to_string()
            },
            device_name,
            enrollment_code,
            sync_interval_secs,
            tls_verify,
            connection_status,
            last_sync_time,
            policy_version,
        })
    }

    /// Update centralized server settings or initiate enrollment
    pub fn update_server_config(
        &self,
        req: UpdateServerConfigRequest,
    ) -> Result<ServerConfig, LocalAdminError> {
        if let Some(url) = &req.server_url {
            let trimmed = url.trim();
            if !trimmed.starts_with("http://") && !trimmed.starts_with("https://") {
                return Err(LocalAdminError::PolicyInvalid(
                    "Server URL must start with http:// or https://".to_string(),
                ));
            }
            self.storage.set_meta("server_url", trimmed)?;
        }

        if let Some(tenant) = &req.tenant_id {
            self.storage.set_meta("enrolled_tenant_id", tenant.trim())?;
        }

        if let Some(name) = &req.device_name {
            self.storage.set_meta("device_name", name.trim())?;
        }

        if let Some(interval) = req.sync_interval_secs {
            let clamped = interval.clamp(10, 86400);
            self.storage
                .set_meta("sync_interval_secs", &clamped.to_string())?;
        }

        if let Some(tls) = req.tls_verify {
            self.storage
                .set_meta("tls_verify", if tls { "true" } else { "false" })?;
        }

        if let Some(code) = &req.enrollment_code {
            let trimmed = code.trim();
            if !trimmed.is_empty() {
                self.storage.set_meta("enrollment_code", trimmed)?;
                let dev_id = format!("dev-{}", trimmed);
                self.storage.set_meta("enrolled_device_id", &dev_id)?;
                self.storage
                    .set_meta("last_sync_time", &Utc::now().to_rfc3339())?;
            }
        }

        if let Some(mode) = &req.operating_mode {
            if mode == "StandaloneFree" {
                self.storage.set_meta("enrolled_device_id", "")?;
                self.storage.set_meta("enrollment_code", "")?;
            }
        }

        self.get_server_config()
    }

    /// Force immediate sync timestamp update
    pub fn trigger_sync(&self) -> Result<String, LocalAdminError> {
        let now_str = Utc::now().to_rfc3339();
        self.storage.set_meta("last_sync_time", &now_str)?;
        let cur_ver = self
            .storage
            .get_meta("policy_version")?
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(1);
        self.storage
            .set_meta("policy_version", &(cur_ver + 1).to_string())?;
        Ok(now_str)
    }

    /// Check whether monitor mode is active (observe & log only, do not block).
    pub fn is_monitor_mode(&self) -> bool {
        self.storage
            .get_meta("monitor_mode")
            .ok()
            .flatten()
            .map(|s| s == "true" || s == "1")
            .unwrap_or(false)
    }

    /// Enable or disable monitor mode.
    pub fn set_monitor_mode(&self, enabled: bool) -> Result<(), LocalAdminError> {
        self.storage
            .set_meta("monitor_mode", if enabled { "true" } else { "false" })?;
        Ok(())
    }

    /// Check whether silent monitoring mode is active (suppress all toast popups).
    pub fn is_silent_mode(&self) -> bool {
        self.storage
            .get_meta("silent_mode")
            .ok()
            .flatten()
            .map(|s| s == "true" || s == "1")
            .unwrap_or(false)
    }

    /// Enable or disable silent monitoring mode.
    pub fn set_silent_mode(&self, enabled: bool) -> Result<(), LocalAdminError> {
        self.storage
            .set_meta("silent_mode", if enabled { "true" } else { "false" })?;
        Ok(())
    }

    /// Retrieve configured DNS blocking action (e.g. "nxdomain", "refused", "zero_ip").
    pub fn get_dns_block_action(&self) -> String {
        self.storage
            .get_meta("dns_block_action")
            .ok()
            .flatten()
            .unwrap_or_else(|| "nxdomain".to_string())
    }

    /// Set DNS blocking action.
    pub fn set_dns_block_action(&self, action: &str) -> Result<(), LocalAdminError> {
        self.storage.set_meta("dns_block_action", action)?;
        Ok(())
    }

    /// Create complete backup snapshot payload with SHA-256 integrity checksum.
    pub fn create_backup(&self) -> Result<BackupPayload, LocalAdminError> {
        let now = Utc::now().to_rfc3339();
        let rules = self.storage.list_local_rules()?;
        let server_cfg = self.get_server_config()?;
        let monitor_mode = self.is_monitor_mode();
        let silent_mode = self.is_silent_mode();

        let settings = BackupSettings {
            operating_mode: server_cfg.operating_mode,
            server_url: server_cfg.server_url,
            tenant_id: server_cfg.tenant_id,
            device_name: server_cfg.device_name,
            sync_interval_secs: server_cfg.sync_interval_secs,
            tls_verify: server_cfg.tls_verify,
            monitor_mode,
            silent_mode,
        };

        let settings_json = serde_json::to_string(&settings)
            .map_err(|e| LocalAdminError::Backup(format!("Serialize settings error: {}", e)))?;
        let rules_json = serde_json::to_string(&rules)
            .map_err(|e| LocalAdminError::Backup(format!("Serialize rules error: {}", e)))?;

        let format_version = "1.0".to_string();
        let app_version = env!("CARGO_PKG_VERSION").to_string();
        let device_id = server_cfg.device_id;

        let checksum = BackupPayload::compute_checksum(
            &format_version,
            &now,
            &device_id,
            &settings_json,
            &rules_json,
        );

        Ok(BackupPayload {
            format_version,
            created_at: now,
            app_version,
            device_id,
            settings,
            rules,
            checksum,
        })
    }

    /// Restore configuration and rules from a backup payload after validating checksum.
    pub fn restore_backup(&self, payload: BackupPayload) -> Result<RestoreResult, LocalAdminError> {
        if payload.format_version != "1.0" {
            return Err(LocalAdminError::Backup(format!(
                "Phiên bản định dạng sao lưu không được hỗ trợ: {}",
                payload.format_version
            )));
        }

        if !payload.verify_checksum() {
            return Err(LocalAdminError::Backup(
                "Tập tin sao lưu bị hỏng hoặc đã bị chỉnh sửa trái phép (Sai lệch mã checksum SHA-256)".to_string(),
            ));
        }

        self.set_monitor_mode(payload.settings.monitor_mode)?;
        let _ = self.set_silent_mode(payload.settings.silent_mode);
        self.storage
            .set_meta("operating_mode", &payload.settings.operating_mode)?;
        self.storage
            .set_meta("server_url", &payload.settings.server_url)?;
        self.storage
            .set_meta("enrolled_tenant_id", &payload.settings.tenant_id)?;
        self.storage
            .set_meta("device_name", &payload.settings.device_name)?;
        self.storage.set_meta(
            "sync_interval_secs",
            &payload.settings.sync_interval_secs.to_string(),
        )?;
        self.storage.set_meta(
            "tls_verify",
            if payload.settings.tls_verify {
                "true"
            } else {
                "false"
            },
        )?;

        let mut restored_count = 0;
        for rule in &payload.rules {
            self.storage.upsert_local_rule(rule)?;
            restored_count += 1;
        }

        let now = Utc::now().to_rfc3339();
        self.storage.set_meta("last_restore_time", &now)?;

        Ok(RestoreResult {
            success: true,
            rules_restored: restored_count,
            message: format!(
                "Khôi phục thành công {} quy tắc và toàn bộ cấu hình hệ thống.",
                restored_count
            ),
        })
    }

    /// Retrieve consolidated activity monitoring summary including total screen time, apps used with timestamps, and web domains visited.
    pub fn get_activity_summary(
        &self,
        active_app: &str,
    ) -> Result<ActivitySummaryResponse, LocalAdminError> {
        let now = Utc::now();
        let date_str = now.format("%Y-%m-%d").to_string();

        // 1. Grouped Application Usage Report
        let grouped_apps = self
            .get_grouped_app_usage(active_app, None, None)
            .unwrap_or_default();

        // 2. Raw App Activities Segments
        let raw_segments = self.storage.list_usage_segments(100).unwrap_or_default();
        let mut app_activities = Vec::new();
        let mut total_duration_minutes: u32 = 0;

        if !raw_segments.is_empty() {
            for (idx, seg) in raw_segments.iter().enumerate() {
                let duration_mins = (seg.duration_ms / 60_000).max(1) as u32;
                total_duration_minutes += duration_mins;
                let (title, cat) = classify_app(&seg.rule_id);
                let end_time = seg.start_utc + chrono::Duration::milliseconds(seg.duration_ms);
                let duration_formatted = if duration_mins >= 60 {
                    format!("{}h {:02}m", duration_mins / 60, duration_mins % 60)
                } else {
                    format!("{} phút", duration_mins)
                };

                app_activities.push(AppActivityItem {
                    id: format!("app-seg-{}", idx + 1),
                    app_name: seg.rule_id.clone(),
                    app_title: title.to_string(),
                    category: cat.to_string(),
                    start_time: seg.start_utc,
                    end_time,
                    duration_minutes: duration_mins,
                    duration_formatted,
                    status: if duration_mins > 60 {
                        "QuotaWarning".to_string()
                    } else {
                        "Normal".to_string()
                    },
                });
            }
        }

        // If raw_segments is empty but grouped_apps has time, use grouped_apps sum
        if total_duration_minutes == 0 && !grouped_apps.is_empty() {
            total_duration_minutes = grouped_apps.iter().map(|a| a.total_duration_minutes).sum();
        }

        // 2. Web Activities
        let raw_logs = self.storage.list_recent_query_logs(200).unwrap_or_default();
        let mut web_activities = Vec::new();
        let mut blocked_count = 0;
        let mut monitored_count = 0;

        if !raw_logs.is_empty() {
            for (idx, log) in raw_logs.iter().enumerate() {
                let cat = classify_domain(&log.domain);
                if log.action == "BLOCK" {
                    blocked_count += 1;
                } else if log.action == "MONITOR" || self.is_monitor_mode() {
                    monitored_count += 1;
                }
                web_activities.push(WebActivityItem {
                    id: idx + 1,
                    domain: log.domain.clone(),
                    category: cat.to_string(),
                    action: log.action.clone(),
                    reason: log.reason.clone(),
                    occurred_at: log.occurred_at,
                    // DNS does not expose the originating process. Keep attribution
                    // honest until ETW/WFP correlation is implemented.
                    source_app: "unknown".to_string(),
                });
            }
        }

        // 3. Unified Timeline
        let mut unified_timeline = Vec::new();
        for app in &app_activities {
            unified_timeline.push(UnifiedTimelineItem {
                id: format!("tl-{}", app.id),
                item_type: "app".to_string(),
                title: app.app_title.clone(),
                subtitle: format!("{} • Sử dụng {}", app.category, app.duration_formatted),
                category: app.category.clone(),
                action: if app.status == "Active" {
                    "ACTIVE".to_string()
                } else {
                    "ALLOW".to_string()
                },
                timestamp: app.start_time,
                duration_minutes: app.duration_minutes,
            });
        }

        for web in &web_activities {
            unified_timeline.push(UnifiedTimelineItem {
                id: format!("tl-web-{}", web.id),
                item_type: "web".to_string(),
                title: web.domain.clone(),
                subtitle: format!("{} • Qua {}", web.category, web.source_app),
                category: web.category.clone(),
                action: web.action.clone(),
                timestamp: web.occurred_at,
                duration_minutes: 0,
            });
        }

        // 3b. Keyword Violations
        let raw_violations = self.storage.list_keyword_violations(50).unwrap_or_default();
        let keyword_violations_list: Vec<KeywordViolationDto> = raw_violations
            .into_iter()
            .map(|v| KeywordViolationDto {
                id: v.id,
                timestamp: v.timestamp_utc,
                app_name: v.app_name,
                keyword_matched: v.keyword_matched,
                category: v.category,
                severity: v.severity,
                snippet_masked: v.snippet_masked,
            })
            .collect();

        for kv in &keyword_violations_list {
            unified_timeline.push(UnifiedTimelineItem {
                id: format!("tl-kw-{}", kv.id),
                item_type: "keyword".to_string(),
                title: format!("Từ khóa nhạy cảm: \"{}\"", kv.keyword_matched),
                subtitle: format!("{} • Ứng dụng {}", kv.category, kv.app_name),
                category: kv.category.clone(),
                action: "VIOLATION".to_string(),
                timestamp: kv.timestamp,
                duration_minutes: 0,
            });
        }

        // Sort unified timeline descending by timestamp
        unified_timeline.sort_by_key(|b| std::cmp::Reverse(b.timestamp));

        let hours = total_duration_minutes / 60;
        let mins = total_duration_minutes % 60;
        let total_screen_time_formatted = if hours > 0 {
            format!("{} giờ {:02} phút", hours, mins)
        } else {
            format!("{} phút", mins)
        };

        let keyword_violations_count = keyword_violations_list.len();

        Ok(ActivitySummaryResponse {
            date: date_str,
            total_screen_time_minutes: total_duration_minutes,
            total_screen_time_formatted,
            active_apps_count: if grouped_apps.is_empty() {
                app_activities.len()
            } else {
                grouped_apps.len()
            },
            total_web_visits_count: web_activities.len(),
            blocked_events_count: blocked_count,
            monitored_events_count: monitored_count,
            keyword_violations_count,
            app_activities,
            grouped_apps,
            web_activities,
            keyword_violations: keyword_violations_list,
            unified_timeline,
        })
    }

    /// Retrieve application usage aggregated and grouped by application name,
    /// calculating the total usage duration for each individual app.
    pub fn get_grouped_app_usage(
        &self,
        active_app: &str,
        start_time: Option<&str>,
        end_time: Option<&str>,
    ) -> Result<Vec<AppUsageSummary>, LocalAdminError> {
        let db_groups = self
            .storage
            .get_app_usage_grouped(start_time, end_time)
            .unwrap_or_default();

        let total_screen_time_ms: i64 = db_groups.iter().map(|g| g.total_duration_ms).sum();

        let mut list = Vec::new();
        for g in db_groups {
            let duration_mins = (g.total_duration_ms / 60_000).max(1) as u32;
            let (title, cat) = classify_app(&g.app_name);
            let duration_formatted = if duration_mins >= 60 {
                format!("{}h {:02}m", duration_mins / 60, duration_mins % 60)
            } else {
                format!("{} phút", duration_mins)
            };
            let is_active_now =
                !active_app.is_empty() && g.app_name.eq_ignore_ascii_case(active_app);
            let percentage = if total_screen_time_ms > 0 {
                ((g.total_duration_ms as f64 / total_screen_time_ms as f64) * 100.0) as f32
            } else {
                0.0
            };

            let status = if is_active_now {
                "Active".to_string()
            } else if duration_mins > 60 {
                "QuotaWarning".to_string()
            } else {
                "Normal".to_string()
            };

            list.push(AppUsageSummary {
                app_name: g.app_name,
                app_title: title.to_string(),
                category: cat.to_string(),
                total_duration_minutes: duration_mins,
                total_duration_formatted: duration_formatted,
                percentage,
                session_count: g.session_count,
                first_seen_at: g.first_seen_utc,
                last_seen_at: g.last_seen_utc,
                is_active_now,
                status,
            });
        }

        // If list is empty but we have an active_app, show it
        if list.is_empty() && !active_app.is_empty() && active_app != "unknown" {
            let (title, cat) = classify_app(active_app);
            list.push(AppUsageSummary {
                app_name: active_app.to_string(),
                app_title: title.to_string(),
                category: cat.to_string(),
                total_duration_minutes: 1,
                total_duration_formatted: "1 phút".to_string(),
                percentage: 100.0,
                session_count: 1,
                first_seen_at: Utc::now(),
                last_seen_at: Utc::now(),
                is_active_now: true,
                status: "Active".to_string(),
            });
        }

        Ok(list)
    }

    // Keyboard & Keyword Monitoring APIs
    pub fn is_keyboard_monitoring_enabled(&self) -> bool {
        self.storage
            .get_meta("keyboard_monitoring_enabled")
            .unwrap_or(None)
            .map(|s| s != "false")
            .unwrap_or(true)
    }

    pub fn set_keyboard_monitoring_enabled(&self, enabled: bool) -> Result<(), LocalAdminError> {
        self.storage.set_meta(
            "keyboard_monitoring_enabled",
            if enabled { "true" } else { "false" },
        )?;
        Ok(())
    }

    pub fn get_keyword_config(&self) -> Result<KeywordConfigResponse, LocalAdminError> {
        let keyboard_monitoring_enabled = self.is_keyboard_monitoring_enabled();
        let violations_today_count = self.storage.count_keyword_violations_today().unwrap_or(0);

        let mut rules = Vec::new();
        // 1. Default system rules
        for r in famshield_core::keyword::default_keyword_rules() {
            let override_key = format!("kw_disabled_{}", r.id);
            let is_disabled = self.storage.get_meta(&override_key)?.as_deref() == Some("1");
            rules.push(KeywordRuleDto {
                id: r.id,
                pattern: r.pattern,
                category: format!("{:?}", r.category),
                category_name: r.category.display_name().to_string(),
                severity: r.severity.as_str().to_string(),
                enabled: !is_disabled,
                is_custom: false,
            });
        }

        // 2. Custom rules from storage
        let custom_list = self.storage.list_custom_keywords().unwrap_or_default();
        for c in custom_list {
            rules.push(KeywordRuleDto {
                id: c.id,
                pattern: c.pattern,
                category: c.category.clone(),
                category_name: if c.category == "Custom" {
                    "Tùy chỉnh phụ huynh".to_string()
                } else {
                    c.category
                },
                severity: c.severity,
                enabled: c.enabled,
                is_custom: true,
            });
        }

        // 3. Recent violations
        let raw_violations = self.storage.list_keyword_violations(30).unwrap_or_default();
        let recent_violations = raw_violations
            .into_iter()
            .map(|v| KeywordViolationDto {
                id: v.id,
                timestamp: v.timestamp_utc,
                app_name: v.app_name,
                keyword_matched: v.keyword_matched,
                category: v.category,
                severity: v.severity,
                snippet_masked: v.snippet_masked,
            })
            .collect();

        let total_rules_count = rules.len();

        Ok(KeywordConfigResponse {
            keyboard_monitoring_enabled,
            total_rules_count,
            violations_today_count,
            rules,
            recent_violations,
        })
    }

    pub fn add_custom_keyword(
        &self,
        req: AddKeywordRequest,
    ) -> Result<KeywordRuleDto, LocalAdminError> {
        let trimmed = req.pattern.trim();
        if trimmed.is_empty() {
            return Err(LocalAdminError::PolicyInvalid(
                "Từ khóa không được để trống".into(),
            ));
        }

        let hash_str = format!("{:x}", sha2::Sha256::digest(trimmed.as_bytes()));
        let id = format!("custom_kw_{}", &hash_str[0..12]);
        let cat = req
            .category
            .unwrap_or_else(|| "Tùy chỉnh phụ huynh".to_string());
        let sev = req.severity.unwrap_or_else(|| "medium".to_string());

        let record = famshield_storage::CustomKeywordRecord {
            id: id.clone(),
            pattern: trimmed.to_string(),
            category: cat.clone(),
            severity: sev.clone(),
            enabled: true,
            created_at: Utc::now(),
        };

        self.storage.upsert_custom_keyword(&record)?;

        Ok(KeywordRuleDto {
            id,
            pattern: trimmed.to_string(),
            category: cat.clone(),
            category_name: cat,
            severity: sev,
            enabled: true,
            is_custom: true,
        })
    }

    pub fn delete_custom_keyword(&self, id: &str) -> Result<bool, LocalAdminError> {
        let deleted = self.storage.delete_custom_keyword(id)?;
        Ok(deleted)
    }

    pub fn toggle_keyword(&self, id: &str, enabled: bool) -> Result<bool, LocalAdminError> {
        if id.starts_with("custom_kw_") {
            self.storage.toggle_custom_keyword(id, enabled)?;
            Ok(true)
        } else {
            let override_key = format!("kw_disabled_{}", id);
            self.storage
                .set_meta(&override_key, if enabled { "0" } else { "1" })?;
            Ok(true)
        }
    }
}

/// Asynchronously probe server reachability and measure round-trip latency
pub async fn probe_server_connection(server_url: &str) -> (bool, u64, String) {
    let trimmed = server_url.trim();
    if !trimmed.starts_with("http://") && !trimmed.starts_with("https://") {
        return (
            false,
            0,
            "URL phải bắt đầu bằng http:// hoặc https://".to_string(),
        );
    }

    let (is_https, rest) = if let Some(stripped) = trimmed.strip_prefix("https://") {
        (true, stripped)
    } else if let Some(stripped) = trimmed.strip_prefix("http://") {
        (false, stripped)
    } else {
        return (false, 0, "URL không hợp lệ".to_string());
    };

    let host_port_part = rest.split('/').next().unwrap_or("");
    let (host, port) = if let Some((h, p)) = host_port_part.split_once(':') {
        (
            h,
            p.parse::<u16>().unwrap_or(if is_https { 443 } else { 80 }),
        )
    } else {
        (host_port_part, if is_https { 443 } else { 80 })
    };

    if host.is_empty() {
        return (false, 0, "Địa chỉ máy chủ trống".to_string());
    }

    let addr = format!("{}:{}", host, port);
    let start = std::time::Instant::now();
    let connect_res = tokio::time::timeout(
        std::time::Duration::from_millis(3000),
        tokio::net::TcpStream::connect(&addr),
    )
    .await;

    let latency = start.elapsed().as_millis() as u64;

    match connect_res {
        Ok(Ok(_)) => (
            true,
            latency,
            format!("Kết nối thành công tới {} ({}ms)", addr, latency),
        ),
        Ok(Err(e)) => (
            false,
            latency,
            format!("Không thể kết nối tới {}: {}", addr, e),
        ),
        Err(_) => (
            false,
            3000,
            format!("Kết nối tới {} quá thời gian (Timeout 3s)", addr),
        ),
    }
}

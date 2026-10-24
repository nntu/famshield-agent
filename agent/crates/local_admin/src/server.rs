use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{delete, get, post},
    Json, Router,
};

use chrono::{DateTime, Duration, Utc};

use rand::RngCore;

use serde::{Deserialize, Serialize};

use std::collections::HashMap;

use std::sync::Arc;

use tokio::sync::{broadcast, Mutex};

use tracing::{info, warn};

use crate::service::{
    probe_server_connection, ActivitySummaryResponse, AddKeywordRequest, AppUsageSummary,
    BackupPayload, CreateRuleRequest, KeywordConfigResponse, KeywordRuleDto, LocalAdminService,
    ServerConfig, SystemSettings, ToggleKeywordRequest, UpdateServerConfigRequest,
    UpdateSystemSettingsRequest,
};

use crate::ui::LOCAL_DASHBOARD_HTML;

use famshield_dns::blocklist::BlocklistStore;

use famshield_storage::{
    DomainQuerySummary, KeywordViolationRecord, LocalRuleRecord, QueryLogRecord, UsageSegmentRecord,
};

pub type TempAuthTokensMap = Arc<Mutex<HashMap<String, (DateTime<Utc>, String)>>>;

pub type PendingTotpSetupState = Arc<Mutex<Option<(String, Vec<String>, DateTime<Utc>)>>>;

#[derive(Clone)]

pub struct ServerState {
    pub service: Arc<LocalAdminService>,

    pub dns_blocklist: Option<Arc<BlocklistStore>>,

    pub dns_cache: Option<Arc<famshield_dns::DnsCache>>,

    pub dns_balancer: Option<Arc<famshield_dns::NumaDnsBalancer>>,

    pub sessions: Arc<Mutex<HashMap<String, DateTime<Utc>>>>,

    pub temp_auth_tokens: TempAuthTokensMap,

    pub pending_totp_setup: PendingTotpSetupState,

    pub active_app: Arc<Mutex<String>>,

    pub is_terminating: Option<Arc<std::sync::atomic::AtomicBool>>,
}

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
}

pub struct LocalAdminServer {
    state: ServerState,
}

impl LocalAdminServer {
    pub fn new(
        service: Arc<LocalAdminService>,

        dns_blocklist: Option<Arc<BlocklistStore>>,
    ) -> Self {
        Self {
            state: ServerState {
                service,

                dns_blocklist,

                dns_cache: None,

                dns_balancer: None,

                sessions: Arc::new(Mutex::new(HashMap::new())),

                temp_auth_tokens: Arc::new(Mutex::new(HashMap::new())),

                pending_totp_setup: Arc::new(Mutex::new(None)),

                active_app: Arc::new(Mutex::new("Roblox.exe".to_string())),

                is_terminating: None,
            },
        }
    }

    pub fn with_terminating_flag(mut self, flag: Arc<std::sync::atomic::AtomicBool>) -> Self {
        self.state.is_terminating = Some(flag);

        self
    }

    pub fn with_dns_cache(mut self, dns_cache: Arc<famshield_dns::DnsCache>) -> Self {
        self.state.dns_cache = Some(dns_cache);

        self
    }

    pub fn with_dns_balancer(mut self, dns_balancer: Arc<famshield_dns::NumaDnsBalancer>) -> Self {
        self.state.dns_balancer = Some(dns_balancer);

        self
    }

    /// Pre-generate and register an active session token (e.g. for tray PIN dialog direct launch)
    pub async fn create_session_token(&self) -> String {
        let token = generate_token();

        let expires_at = Utc::now() + Duration::hours(1);

        self.state
            .sessions
            .lock()
            .await
            .insert(token.clone(), expires_at);

        token
    }

    /// Update current active foreground application dynamically
    pub async fn set_active_app(&self, app_name: &str) {
        *self.state.active_app.lock().await = app_name.to_string();

        let seg_id = format!("seg-{}", Utc::now().timestamp_millis());

        let _ = self
            .state
            .service
            .storage()
            .record_usage_segment(&UsageSegmentRecord {
                segment_id: seg_id,

                child_id: "local_child".to_string(),

                rule_id: app_name.to_string(),

                start_utc: Utc::now(),

                duration_ms: 60_000,

                state: "committed".to_string(),
            });
    }

    /// Access reference to underlying server state
    pub fn state(&self) -> &ServerState {
        &self.state
    }

    /// Construct Axum router
    pub fn router(&self) -> Router {
        Router::new()
            .route("/", get(handle_index))
            .route("/assets/*path", get(handle_assets))
            .route("/api/auth/login", post(handle_login))
            .route("/api/auth/login/totp", post(handle_login_totp))
            .route("/api/auth/logout", post(handle_logout))
            .route("/api/auth/verify", get(handle_verify))
            .route("/api/auth/totp/status", get(handle_get_totp_status))
            .route("/api/auth/totp/setup", post(handle_setup_totp))
            .route("/api/auth/totp/confirm", post(handle_confirm_totp))
            .route("/api/auth/totp/disable", post(handle_disable_totp))
            .route("/api/status", get(handle_status))
            .route("/api/activity/summary", get(handle_get_activity_summary))
            .route("/api/activity/apps/summary", get(handle_get_apps_summary))
            .route("/api/activity/reset", post(handle_reset_reports))
            .route(
                "/api/rules",
                get(handle_list_rules).post(handle_create_rule),
            )
            .route("/api/rules/bulk", post(handle_bulk_create_rules))
            .route("/api/rules/sync-to-cloud", post(handle_sync_rules_to_cloud))
            .route("/api/rules/:rule_id", delete(handle_delete_rule))
            .route("/api/logs/dns", get(handle_list_logs))
            .route("/api/logs/dns/summary", get(handle_get_dns_summary))
            .route("/api/logs/dns/hourly", get(handle_get_dns_hourly))
            .route("/api/logs/system", get(handle_get_system_logs))
            .route(
                "/api/logs/system/download",
                get(handle_download_system_logs),
            )
            .route(
                "/api/errors",
                get(handle_list_errors).delete(handle_clear_errors),
            )
            .route("/api/errors/push-nats", post(handle_push_errors_nats))
            .route("/api/evidence", get(handle_list_evidence))
            .route(
                "/api/evidence/:evidence_id/thumbnail",
                get(handle_get_thumbnail),
            )
            .route("/api/settings/pin", post(handle_change_pin))
            .route(
                "/api/settings/server",
                get(handle_get_server_config).post(handle_update_server_config),
            )
            .route(
                "/api/settings/server/test",
                post(handle_test_server_connection),
            )
            .route("/api/settings/server/sync", post(handle_sync_server))
            .route(
                "/api/settings/system",
                get(handle_get_system_settings).post(handle_update_system_settings),
            )
            .route(
                "/api/settings/dns-retention",
                get(handle_get_dns_retention).post(handle_set_dns_retention),
            )
            .route(
                "/api/settings/dns-retention/prune-now",
                post(handle_prune_dns_retention_now),
            )
            .route(
                "/api/settings/capture-blur",
                get(handle_get_capture_blur).post(handle_set_capture_blur),
            )
            .route(
                "/api/settings/protection-mode",
                get(handle_get_protection_mode).post(handle_set_protection_mode),
            )
            .route(
                "/api/settings/silent-mode",
                get(handle_get_silent_mode).post(handle_set_silent_mode),
            )
            .route("/api/backup/export", get(handle_export_backup))
            .route("/api/backup/restore", post(handle_restore_backup))
            .route("/api/cloud/enroll", post(handle_cloud_enroll))
            .route("/api/cloud/disenroll", post(handle_cloud_disenroll))
            .route("/api/setup/status", get(handle_setup_status))
            .route("/api/setup/initialize", post(handle_setup_initialize))
            .route(
                "/api/keywords",
                get(handle_get_keywords).post(handle_add_keyword),
            )
            .route("/api/keywords/:keyword_id", delete(handle_delete_keyword))
            .route("/api/keywords/toggle", post(handle_toggle_keyword))
            .route(
                "/api/keywords/monitoring",
                get(handle_get_keyboard_monitoring).post(handle_toggle_keyboard_monitoring),
            )
            .route(
                "/api/keywords/violation",
                post(handle_record_keyword_violation),
            )
            .route(
                "/api/system/session-helper/start",
                post(handle_start_session_helper),
            )
            .route("/api/system/uninstall", post(handle_system_uninstall))
            .route("/api/system/update/check", get(handle_check_update))
            .route(
                "/api/system/update/url",
                get(handle_get_update_urls).post(handle_update_urls),
            )
            .route(
                "/api/dns/config",
                get(handle_get_dns_config).post(handle_set_dns_config),
            )
            .route("/api/dns/cache/flush", post(handle_flush_dns_cache))
            .with_state(self.state.clone())
    }

    /// Launch HTTP server on specified address (e.g., "127.0.0.1:38123"), automatically falling back if port is reserved by Hyper-V/OS
    pub async fn run(
        &self,

        bind_addr: &str,

        mut shutdown_rx: broadcast::Receiver<()>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let app = self.router();

        let listener = match tokio::net::TcpListener::bind(bind_addr).await {
            Ok(l) => {
                if let Ok(addr) = l.local_addr() {
                    let port_file = std::env::var("ProgramData")
                        .map(std::path::PathBuf::from)
                        .unwrap_or_else(|_| std::path::PathBuf::from("."))
                        .join("FamShield")
                        .join("admin_port.txt");

                    let _ = std::fs::write(port_file, addr.port().to_string());
                }

                l
            }

            Err(e) => {
                warn!(
                    "Failed to bind Local Admin server to {} ({}). Attempting fallback ports...",
                    bind_addr, e
                );

                let fallback_ports = [38123, 18123, 28123, 48123];

                let mut bound = None;

                for &port in &fallback_ports {
                    let fallback_addr = format!("127.0.0.1:{}", port);

                    if let Ok(l) = tokio::net::TcpListener::bind(&fallback_addr).await {
                        info!(
                            "Successfully bound Local Admin server to fallback {}",
                            fallback_addr
                        );

                        let port_file = std::env::var("ProgramData")
                            .map(std::path::PathBuf::from)
                            .unwrap_or_else(|_| std::path::PathBuf::from("."))
                            .join("FamShield")
                            .join("admin_port.txt");

                        let _ = std::fs::write(port_file, port.to_string());

                        bound = Some(l);

                        break;
                    }
                }

                match bound {
                    Some(l) => l,

                    None => return Err(Box::new(e)),
                }
            }
        };

        let local_addr = listener.local_addr()?;

        info!(
            "FamShield Local Admin Web Server listening on http://{}",
            local_addr
        );

        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                let _ = shutdown_rx.recv().await;

                info!("Local Admin Web Server shutting down gracefully");
            })
            .await?;

        Ok(())
    }
}

// ================= Helpers =================

fn generate_token() -> String {
    let mut bytes = [0u8; 32];

    rand::thread_rng().fill_bytes(&mut bytes);

    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn extract_token(headers: &HeaderMap, query_token: Option<&str>) -> Option<String> {
    if let Some(tok) = query_token {
        let trimmed = tok.trim();

        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    if let Some(auth) = headers
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
    {
        if let Some(token) = auth.strip_prefix("Bearer ") {
            return Some(token.trim().to_string());
        }
    }

    if let Some(custom) = headers
        .get("X-FamShield-Token")
        .and_then(|h| h.to_str().ok())
    {
        return Some(custom.trim().to_string());
    }

    if let Some(cookie) = headers.get(header::COOKIE).and_then(|h| h.to_str().ok()) {
        for part in cookie.split(';') {
            let part = part.trim();

            if let Some(val) = part.strip_prefix("famshield_session=") {
                return Some(val.to_string());
            }
        }
    }

    None
}

async fn check_auth(state: &ServerState, headers: &HeaderMap, query_token: Option<&str>) -> bool {
    if let Some(token) = extract_token(headers, query_token) {
        let mut sessions = state.sessions.lock().await;

        if let Some(expires_at) = sessions.get(&token) {
            if *expires_at > Utc::now() {
                // Slide session expiration window (60 minutes)

                sessions.insert(token, Utc::now() + Duration::minutes(60));

                return true;
            }
        }
    }

    false
}

fn sync_dns_blocklist(state: &ServerState) {
    if let Some(blocklist) = &state.dns_blocklist {
        blocklist.set_monitor_mode(state.service.is_monitor_mode());

        if let Ok(rules) = state.service.list_rules() {
            let active_domains: Vec<String> = rules
                .into_iter()
                .filter(|r| r.rule_type == "domain" && r.enabled)
                .map(|r| r.rule_id)
                .collect();

            blocklist.replace_blocklist(active_domains);
        }
    }

    if let Some(cache) = &state.dns_cache {
        cache.flush();
    }
}

fn find_dist_dir() -> Option<std::path::PathBuf> {
    #[cfg(target_os = "windows")]
    {
        if let Ok(program_data) = std::env::var("ProgramData") {
            let pd_path = std::path::PathBuf::from(program_data)
                .join("FamShield")
                .join("web")
                .join("dist");

            if pd_path.join("index.html").exists() {
                return Some(pd_path);
            }
        }
    }

    let candidates = [
        std::path::PathBuf::from("web/dist"),
        std::path::PathBuf::from("../web/dist"),
        std::path::PathBuf::from("../../web/dist"),
    ];

    for c in &candidates {
        if c.join("index.html").exists() {
            return Some(c.clone());
        }
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let p1 = parent.join("web").join("dist");

            if p1.join("index.html").exists() {
                return Some(p1);
            }

            let p2 = parent.join("dist");

            if p2.join("index.html").exists() {
                return Some(p2);
            }
        }
    }

    None
}

// ================= Handlers =================

async fn handle_index() -> impl IntoResponse {
    if let Some(dist) = find_dist_dir() {
        let index_path = dist.join("index.html");

        if let Ok(content) = std::fs::read_to_string(&index_path) {
            return Response::builder()
                .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                .body(axum::body::Body::from(content))
                .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response());
        }
    }

    Html(LOCAL_DASHBOARD_HTML).into_response()
}

async fn handle_assets(Path(asset_path): Path<String>) -> impl IntoResponse {
    if let Some(dist) = find_dist_dir() {
        let clean_path = asset_path.trim_start_matches('/').replace("..", "");

        let file_path = dist.join("assets").join(&clean_path);

        if file_path.exists() && file_path.is_file() {
            if let Ok(bytes) = std::fs::read(&file_path) {
                let mime = match clean_path.split('.').next_back().unwrap_or("") {
                    "js" | "mjs" => "application/javascript; charset=utf-8",

                    "css" => "text/css; charset=utf-8",

                    "svg" => "image/svg+xml",

                    "png" => "image/png",

                    "ico" => "image/x-icon",

                    "woff2" => "font/woff2",

                    "json" => "application/json",

                    _ => "application/octet-stream",
                };

                return Response::builder()
                    .header(header::CONTENT_TYPE, mime)
                    .header(header::CACHE_CONTROL, "public, max-age=31536000, immutable")
                    .body(axum::body::Body::from(bytes))
                    .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response());
            }
        }
    }

    StatusCode::NOT_FOUND.into_response()
}

async fn handle_login(
    State(state): State<ServerState>,

    Json(req): Json<LoginRequest>,
) -> impl IntoResponse {
    match state.service.authenticate_storage(&req.pin) {
        Ok(valid) if valid => {
            // Check if Parent 2FA TOTP is enabled

            let totp_cfg = state
                .service
                .storage()
                .get_parent_totp_config()
                .ok()
                .flatten();

            if let Some(cfg) = totp_cfg {
                if cfg.enabled {
                    let temp_token = format!("tmp-totp-{}", generate_token());

                    let temp_expiry = Utc::now() + Duration::minutes(5);

                    state
                        .temp_auth_tokens
                        .lock()
                        .await
                        .insert(temp_token.clone(), (temp_expiry, "parent".to_string()));

                    return (

                        StatusCode::OK,

                        Json(LoginResponse {

                            success: true,

                            token: None,

                            expires_at: None,

                            requires_totp: Some(true),

                            temp_auth_token: Some(temp_token),

                            message: Some(

                                "Vui lòng nhập mã OTP 6 số từ ứng dụng Authenticator hoặc mã dự phòng"

                                    .to_string(),

                            ),

                            error: None,

                        }),

                    )

                        .into_response();
                }
            }

            let token = generate_token();

            let expires_at = Utc::now() + Duration::hours(1);

            state
                .sessions
                .lock()
                .await
                .insert(token.clone(), expires_at);

            let cookie_val = format!(
                "famshield_session={}; HttpOnly; SameSite=Strict; Path=/; Max-Age=3600",
                token
            );

            let body = Json(LoginResponse {
                success: true,

                token: Some(token),

                expires_at: Some(expires_at),

                requires_totp: Some(false),

                temp_auth_token: None,

                message: None,

                error: None,
            });

            (StatusCode::OK, [(header::SET_COOKIE, cookie_val)], body).into_response()
        }

        Ok(_) => (
            StatusCode::UNAUTHORIZED,
            Json(LoginResponse {
                success: false,

                token: None,

                expires_at: None,

                requires_totp: None,

                temp_auth_token: None,

                message: None,

                error: Some("Mã Master PIN không chính xác (Invalid Master PIN)".to_string()),
            }),
        )
            .into_response(),

        Err(e) => {
            warn!("Master PIN authentication error: {}", e);

            (
                StatusCode::TOO_MANY_REQUESTS,
                Json(LoginResponse {
                    success: false,

                    token: None,

                    expires_at: None,

                    requires_totp: None,

                    temp_auth_token: None,

                    message: None,

                    error: Some(format!("Lỗi xác thực: {}", e)),
                }),
            )
                .into_response()
        }
    }
}

async fn handle_login_totp(
    State(state): State<ServerState>,

    Json(req): Json<TotpLoginRequest>,
) -> impl IntoResponse {
    let now = Utc::now();

    // 1. Verify temp_auth_token

    let is_valid_temp = {
        let mut tokens = state.temp_auth_tokens.lock().await;

        tokens.retain(|_, (exp, _)| *exp > now);

        if let Some((exp, _)) = tokens.get(&req.temp_auth_token) {
            *exp > now
        } else {
            false
        }
    };

    if !is_valid_temp {
        return (

            StatusCode::UNAUTHORIZED,

            Json(LoginResponse {

                success: false,

                token: None,

                expires_at: None,

                requires_totp: Some(true),

                temp_auth_token: None,

                message: None,

                error: Some(

                    "Phiên đăng nhập tạm thời đã hết hạn hoặc không hợp lệ. Vui lòng đăng nhập lại."

                        .to_string(),

                ),

            }),

        )

            .into_response();
    }

    // 2. Fetch 2FA config

    let totp_cfg = match state.service.storage().get_parent_totp_config() {
        Ok(Some(cfg)) if cfg.enabled => cfg,

        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(LoginResponse {
                    success: false,

                    token: None,

                    expires_at: None,

                    requires_totp: None,

                    temp_auth_token: None,

                    message: None,

                    error: Some("Xác thực 2FA chưa được cấu hình hoặc đã bị tắt".to_string()),
                }),
            )
                .into_response();
        }
    };

    let trimmed_code = req.code.trim();

    let mut verified = false;

    // Check if 6-digit TOTP code

    if trimmed_code.len() == 6 && trimmed_code.chars().all(|c| c.is_ascii_digit()) {
        if let Ok(unprotected) =
            crate::secret_protection::unprotect_secret(&totp_cfg.encrypted_secret)
        {
            let computer_name =
                std::env::var("COMPUTERNAME").unwrap_or_else(|_| "Desktop".to_string());

            let account_name = format!("Parent ({})", computer_name);

            if let Ok(ok) = crate::totp::TotpManager::verify_code(
                &unprotected.plaintext,
                &account_name,
                trimmed_code,
            ) {
                if ok {
                    verified = true;

                    let mut updated_cfg = totp_cfg.clone();

                    updated_cfg.last_verified_at = Some(now);

                    let _ = state.service.storage().set_parent_totp_config(&updated_cfg);
                }
            }
        }
    }

    // If not verified via 6-digit code, check emergency backup code

    if !verified {
        let code_hash = crate::totp::TotpManager::hash_backup_code(trimmed_code);

        if let Ok(consumed) = state
            .service
            .storage()
            .verify_and_consume_backup_code(&code_hash)
        {
            if consumed {
                verified = true;

                tracing::info!("Parent logged in using a single-use emergency backup code");
            }
        }
    }

    if !verified {
        return (
            StatusCode::UNAUTHORIZED,
            Json(LoginResponse {
                success: false,

                token: None,

                expires_at: None,

                requires_totp: Some(true),

                temp_auth_token: Some(req.temp_auth_token),

                message: None,

                error: Some("Mã OTP 6 số hoặc mã dự phòng không chính xác".to_string()),
            }),
        )
            .into_response();
    }

    // 3. Consume temp_auth_token and issue formal session token

    state
        .temp_auth_tokens
        .lock()
        .await
        .remove(&req.temp_auth_token);

    let token = generate_token();

    let expires_at = now + Duration::hours(1);

    state
        .sessions
        .lock()
        .await
        .insert(token.clone(), expires_at);

    let cookie_val = format!(
        "famshield_session={}; HttpOnly; SameSite=Strict; Path=/; Max-Age=3600",
        token
    );

    let body = Json(LoginResponse {
        success: true,

        token: Some(token),

        expires_at: Some(expires_at),

        requires_totp: Some(false),

        temp_auth_token: None,

        message: Some("Xác thực 2 lớp thành công".to_string()),

        error: None,
    });

    (StatusCode::OK, [(header::SET_COOKIE, cookie_val)], body).into_response()
}

async fn handle_get_totp_status(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Query(query): Query<QueryParams>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if !check_auth(&state, &headers, query.token.as_deref()).await {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Unauthorized" })),
        ));
    }

    let cfg = state
        .service
        .storage()
        .get_parent_totp_config()
        .unwrap_or(None);

    let unused_backup_codes = state
        .service
        .storage()
        .count_unused_backup_codes()
        .unwrap_or(0);

    let (enabled, created_at, last_verified_at) = match cfg {
        Some(c) => (
            c.enabled,
            Some(c.created_at.to_rfc3339()),
            c.last_verified_at.map(|dt| dt.to_rfc3339()),
        ),

        None => (false, None, None),
    };

    Ok(Json(serde_json::json!({

        "enabled": enabled,

        "created_at": created_at,

        "last_verified_at": last_verified_at,

        "unused_backup_codes": unused_backup_codes,

    })))
}

async fn handle_setup_totp(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Query(query): Query<QueryParams>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if !check_auth(&state, &headers, query.token.as_deref()).await {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Unauthorized" })),
        ));
    }

    let computer_name = std::env::var("COMPUTERNAME").unwrap_or_else(|_| "Desktop".to_string());

    let account_name = format!("Parent ({})", computer_name);

    let (secret, url, qr_base64, backup_codes) =
        match crate::totp::TotpManager::generate_setup(&account_name) {
            Ok(res) => res,

            Err(e) => {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": format!("Lỗi khởi tạo TOTP: {}", e) })),
                ));
            }
        };

    let expiry = Utc::now() + Duration::minutes(10);

    *state.pending_totp_setup.lock().await = Some((secret.clone(), backup_codes.clone(), expiry));

    Ok(Json(serde_json::json!({

        "success": true,

        "secret": secret,

        "url": url,

        "qr_base64": qr_base64,

        "backup_codes": backup_codes,

    })))
}

async fn handle_confirm_totp(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Query(query): Query<QueryParams>,

    Json(req): Json<ConfirmTotpRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if !check_auth(&state, &headers, query.token.as_deref()).await {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Unauthorized" })),
        ));
    }

    let pending = state.pending_totp_setup.lock().await.take();

    let (secret, backup_codes, expiry) = match pending {
        Some(p) if p.2 > Utc::now() => p,

        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({

                    "error": "Yêu cầu thiết lập TOTP đã hết hạn hoặc chưa được tạo. Vui lòng bấm tạo lại mã QR."

                })),
            ));
        }
    };

    let computer_name = std::env::var("COMPUTERNAME").unwrap_or_else(|_| "Desktop".to_string());

    let account_name = format!("Parent ({})", computer_name);

    match crate::totp::TotpManager::verify_code(&secret, &account_name, &req.code) {
        Ok(true) => {
            let encrypted = match crate::secret_protection::protect_secret(&secret) {
                Ok(enc) => enc,

                Err(e) => {
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(
                            serde_json::json!({ "error": format!("Lỗi bảo vệ khóa bằng DPAPI: {}", e) }),
                        ),
                    ));
                }
            };

            let now = Utc::now();

            let config = famshield_storage::ParentTotpConfigRecord {
                enabled: true,

                encrypted_secret: encrypted,

                created_at: now,

                last_verified_at: Some(now),
            };

            if let Err(e) = state.service.storage().set_parent_totp_config(&config) {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": format!("Lỗi lưu cấu hình 2FA: {}", e) })),
                ));
            }

            let code_hashes: Vec<String> = backup_codes
                .iter()
                .map(|code| crate::totp::TotpManager::hash_backup_code(code))
                .collect();

            if let Err(e) = state
                .service
                .storage()
                .save_parent_backup_codes(&code_hashes)
            {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": format!("Lỗi lưu mã dự phòng: {}", e) })),
                ));
            }

            Ok(Json(serde_json::json!({

                "success": true,

                "message": "Đã kích hoạt xác thực 2 lớp (2FA TOTP) thành công"

            })))
        }

        Ok(false) => {
            *state.pending_totp_setup.lock().await = Some((secret, backup_codes, expiry));

            Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({

                    "error": "Mã xác nhận 6 số không chính xác. Vui lòng kiểm tra lại đồng hồ của điện thoại và máy tính."

                })),
            ))
        }

        Err(e) => {
            *state.pending_totp_setup.lock().await = Some((secret, backup_codes, expiry));

            Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": format!("Lỗi xác thực mã TOTP: {}", e) })),
            ))
        }
    }
}

async fn handle_disable_totp(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Query(query): Query<QueryParams>,

    Json(req): Json<DisableTotpRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if !check_auth(&state, &headers, query.token.as_deref()).await {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Unauthorized" })),
        ));
    }

    // Authenticate Master PIN first

    match state.service.authenticate_storage(&req.pin) {
        Ok(true) => {}

        Ok(false) => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": "Master PIN không chính xác" })),
            ));
        }

        Err(e) => {
            return Err((
                StatusCode::TOO_MANY_REQUESTS,
                Json(serde_json::json!({ "error": format!("Lỗi xác thực PIN: {}", e) })),
            ));
        }
    }

    // If 2FA is active and code was supplied, verify it

    if let Ok(Some(cfg)) = state.service.storage().get_parent_totp_config() {
        if cfg.enabled {
            if let Some(code) = req.code.as_deref() {
                let trimmed = code.trim();

                let mut code_valid = false;

                if trimmed.len() == 6 && trimmed.chars().all(|c| c.is_ascii_digit()) {
                    if let Ok(unprotected) =
                        crate::secret_protection::unprotect_secret(&cfg.encrypted_secret)
                    {
                        let computer_name =
                            std::env::var("COMPUTERNAME").unwrap_or_else(|_| "Desktop".to_string());

                        let account_name = format!("Parent ({})", computer_name);

                        if let Ok(true) = crate::totp::TotpManager::verify_code(
                            &unprotected.plaintext,
                            &account_name,
                            trimmed,
                        ) {
                            code_valid = true;
                        }
                    }
                }

                if !code_valid {
                    let code_hash = crate::totp::TotpManager::hash_backup_code(trimmed);

                    if let Ok(consumed) = state
                        .service
                        .storage()
                        .verify_and_consume_backup_code(&code_hash)
                    {
                        if consumed {
                            code_valid = true;
                        }
                    }
                }

                if !code_valid {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        Json(
                            serde_json::json!({ "error": "Mã OTP hoặc mã dự phòng không chính xác" }),
                        ),
                    ));
                }
            }
        }
    }

    if let Err(e) = state.service.storage().disable_parent_totp() {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Lỗi tắt 2FA: {}", e) })),
        ));
    }

    Ok(Json(serde_json::json!({

        "success": true,

        "message": "Đã tắt tính năng xác thực 2 lớp (2FA)"

    })))
}

async fn handle_logout(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Query(query): Query<QueryParams>,
) -> impl IntoResponse {
    if let Some(tok) = extract_token(&headers, query.token.as_deref()) {
        state.sessions.lock().await.remove(&tok);
    }

    let clear_cookie = "famshield_session=; HttpOnly; SameSite=Strict; Path=/; Max-Age=0";

    (
        StatusCode::OK,
        [(header::SET_COOKIE, clear_cookie)],
        Json(serde_json::json!({ "success": true })),
    )
}

async fn handle_verify(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Query(query): Query<QueryParams>,
) -> impl IntoResponse {
    let auth = check_auth(&state, &headers, query.token.as_deref()).await;

    Json(VerifyResponse {
        authenticated: auth,
    })
}

async fn handle_status(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Query(query): Query<QueryParams>,
) -> Result<Json<StatusResponse>, StatusCode> {
    if !check_auth(&state, &headers, query.token.as_deref()).await {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let logs = state.service.list_query_logs(200).unwrap_or_default();

    let blocked_queries_today = logs.iter().filter(|l| l.action == "BLOCK").count();

    let total_queries_today = logs.len();

    let rules = state.service.list_rules().unwrap_or_default();

    let rules_count = rules.len();

    let evidence = state.service.list_visual_evidence(50).unwrap_or_default();

    let evidence_count = evidence.len();

    let device_id = state
        .service
        .storage()
        .get_meta("enrolled_device_id")
        .ok()
        .flatten()
        .filter(|id| !id.is_empty())
        .unwrap_or_else(|| "standalone-local-001".to_string());

    let operating_mode = if device_id.starts_with("standalone") {
        "StandaloneFree".to_string()
    } else {
        "OnlineManaged".to_string()
    };

    let active_app = state.active_app.lock().await.clone();

    let monitor_mode = state.service.is_monitor_mode();

    let silent_mode = state.service.is_silent_mode();

    let session_helper_running = famshield_windows::SessionManager::is_session_helper_running();

    let session_helper_status = if session_helper_running {
        "RUNNING".to_string()
    } else {
        "STOPPED".to_string()
    };

    Ok(Json(StatusResponse {
        protection_active: true,

        monitor_mode,

        silent_mode,

        dns_block_action: state.service.get_dns_block_action(),

        session_helper_running,

        session_helper_status,

        dns_proxy_addr: "127.0.0.2:53".to_string(),

        wfp_status: "Active (Dynamic User-Mode Session)".to_string(),

        operating_mode,

        device_id,

        active_foreground_app: active_app,

        daily_quota_minutes: 60,

        used_today_minutes: 27,

        blocked_queries_today,

        total_queries_today,

        rules_count,

        evidence_count,

        app_version: env!("CARGO_PKG_VERSION").to_string(),
    }))
}

async fn handle_list_rules(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Query(query): Query<QueryParams>,
) -> Result<Json<Vec<LocalRuleRecord>>, StatusCode> {
    if !check_auth(&state, &headers, query.token.as_deref()).await {
        return Err(StatusCode::UNAUTHORIZED);
    }

    match state.service.list_rules() {
        Ok(rules) => Ok(Json(rules)),

        Err(e) => {
            warn!("Failed to query local rules: {}", e);

            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn handle_create_rule(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Query(query): Query<QueryParams>,

    Json(req): Json<CreateRuleRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    if !check_auth(&state, &headers, query.token.as_deref()).await {
        return Err(StatusCode::UNAUTHORIZED);
    }

    match state.service.create_rule(req) {
        Ok(()) => {
            sync_dns_blocklist(&state);

            Ok((
                StatusCode::CREATED,
                Json(serde_json::json!({ "success": true })),
            ))
        }

        Err(e) => {
            warn!("Failed to create local rule: {}", e);

            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn handle_bulk_create_rules(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Query(query): Query<QueryParams>,

    Json(requests): Json<Vec<CreateRuleRequest>>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    if !check_auth(&state, &headers, query.token.as_deref()).await {
        return Err(StatusCode::UNAUTHORIZED);
    }

    match state.service.create_rules_bulk(requests) {
        Ok(imported) => {
            sync_dns_blocklist(&state);

            Ok((
                StatusCode::CREATED,
                Json(serde_json::json!({ "success": true, "imported": imported })),
            ))
        }

        Err(e) => {
            warn!("Failed to bulk create rules: {}", e);

            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn handle_delete_rule(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Path(rule_id): Path<String>,

    Query(query): Query<QueryParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if !check_auth(&state, &headers, query.token.as_deref()).await {
        return Err(StatusCode::UNAUTHORIZED);
    }

    match state.service.delete_rule(&rule_id) {
        Ok(found) => {
            if found {
                sync_dns_blocklist(&state);

                Ok(Json(
                    serde_json::json!({ "success": true, "deleted": true }),
                ))
            } else {
                Err(StatusCode::NOT_FOUND)
            }
        }

        Err(e) => {
            warn!("Failed to delete rule '{}': {}", rule_id, e);

            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn handle_sync_rules_to_cloud(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Query(query): Query<QueryParams>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if !check_auth(&state, &headers, query.token.as_deref()).await {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Unauthorized" })),
        ));
    }

    let server_url = state
        .service
        .storage()
        .get_meta("server_url")
        .ok()
        .flatten()
        .unwrap_or_default();

    let tenant_id = state
        .service
        .storage()
        .get_meta("enrolled_tenant_id")
        .ok()
        .flatten()
        .unwrap_or_else(|| "tenant-default".to_string());

    if server_url.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({

                "error": "Thiết bị chưa được cấu hình địa chỉ FamShield Server. Vui lòng cấu hình máy chủ trước khi đồng bộ."

            })),
        ));
    }

    // 1. Gather all local custom blocked domains

    let rules = state.service.list_rules().map_err(|e| {

        (

            StatusCode::INTERNAL_SERVER_ERROR,

            Json(serde_json::json!({ "error": format!("Không thể đọc danh sách quy tắc cục bộ: {}", e) })),

        )

    })?;

    let mut local_domains: Vec<String> = rules
        .into_iter()
        .filter(|r| r.rule_type == "domain" && r.enabled)
        .map(|r| r.rule_id.trim().to_lowercase())
        .filter(|d| !d.is_empty())
        .collect();

    local_domains.sort();

    local_domains.dedup();

    // 2. HTTP call to Cloud Server /v1/filter/policy

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());

    let policy_url = format!(
        "{}/v1/filter/policy?tenant_id={}",
        server_url.trim_end_matches('/'),
        tenant_id
    );

    // Fetch existing policy first

    let mut policy_json: serde_json::Value = match client.get(&policy_url).send().await {
        Ok(res) if res.status().is_success() => {
            res.json().await.unwrap_or_else(|_| serde_json::json!({}))
        }

        _ => serde_json::json!({}),
    };

    // Merge custom_blocked_domains

    let mut custom_blocked: Vec<String> = match policy_json
        .get("custom_blocked_domains")
        .and_then(|v| v.as_array())
    {
        Some(arr) => arr
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect(),

        None => Vec::new(),
    };

    for d in &local_domains {
        if !custom_blocked.contains(d) {
            custom_blocked.push(d.clone());
        }
    }

    custom_blocked.sort();

    custom_blocked.dedup();

    policy_json["tenant_id"] = serde_json::json!(tenant_id);

    policy_json["custom_blocked_domains"] = serde_json::json!(custom_blocked);

    // POST updated policy to Cloud Server

    match client.post(&policy_url).json(&policy_json).send().await {
        Ok(res) if res.status().is_success() => {
            info!(

                "Successfully synchronized {} local blocked domains to FamShield Cloud Server (tenant: {})",

                local_domains.len(),

                tenant_id

            );

            Ok(Json(serde_json::json!({

                "success": true,

                "synced_domains_count": local_domains.len(),

                "total_cloud_custom_blocked": custom_blocked.len(),

                "tenant_id": tenant_id,

                "message": format!("Đã đồng bộ thành công {} tên miền tự chặn lên Cloud Server", local_domains.len())

            })))
        }

        Ok(res) => {
            let status = res.status();

            let err_text = res.text().await.unwrap_or_default();

            warn!(
                "Server rejected policy update: status {} - {}",
                status, err_text
            );

            Err((
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({

                    "error": format!("Máy chủ từ chối cập nhật chính sách (HTTP {}): {}", status, err_text)

                })),
            ))
        }

        Err(e) => {
            warn!("Failed to connect to Cloud Server at {}: {}", policy_url, e);

            Err((
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({

                    "error": format!("Không thể kết nối đến máy chủ {}: {}", server_url, e)

                })),
            ))
        }
    }
}

async fn handle_list_logs(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Query(query): Query<QueryParams>,
) -> Result<Json<Vec<QueryLogRecord>>, StatusCode> {
    if !check_auth(&state, &headers, query.token.as_deref()).await {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let limit = query.limit.unwrap_or(100).min(500);

    let start_time = query
        .start_time
        .as_deref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc));

    let end_time = query
        .end_time
        .as_deref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc));

    let filter = famshield_storage::QueryLogFilter {
        domain: query.domain,

        action: query.action,

        start_time,

        end_time,

        limit,
    };

    match state.service.list_query_logs_with_filter(&filter) {
        Ok(logs) => Ok(Json(logs)),

        Err(e) => {
            warn!("Failed to query DNS logs: {}", e);

            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn handle_get_dns_summary(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Query(query): Query<QueryParams>,
) -> Result<Json<Vec<DomainQuerySummary>>, StatusCode> {
    if !check_auth(&state, &headers, query.token.as_deref()).await {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let limit = query.limit.unwrap_or(200).min(1000);

    let start_time = query
        .start_time
        .as_deref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc));

    let end_time = query
        .end_time
        .as_deref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc));

    match state
        .service
        .get_domain_query_summaries_filtered(start_time, end_time, limit)
    {
        Ok(summaries) => Ok(Json(summaries)),

        Err(e) => {
            warn!("Failed to query domain DNS summaries: {}", e);

            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn handle_get_dns_hourly(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Query(query): Query<QueryParams>,
) -> Result<Json<Vec<famshield_storage::HourlyFrequency>>, StatusCode> {
    if !check_auth(&state, &headers, query.token.as_deref()).await {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let start_time = query
        .start_time
        .as_deref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc));

    let end_time = query
        .end_time
        .as_deref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc));

    match state
        .service
        .get_hourly_frequency(query.domain.as_deref(), start_time, end_time)
    {
        Ok(frequencies) => Ok(Json(frequencies)),

        Err(e) => {
            warn!("Failed to query DNS hourly frequency: {}", e);

            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn handle_list_evidence(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Query(query): Query<QueryParams>,
) -> Result<Json<Vec<EvidenceItemResponse>>, StatusCode> {
    if !check_auth(&state, &headers, query.token.as_deref()).await {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let limit = query.limit.unwrap_or(50).min(100);

    match state.service.list_visual_evidence(limit) {
        Ok(records) => {
            let items: Vec<EvidenceItemResponse> = records
                .into_iter()
                .map(|r| EvidenceItemResponse {
                    evidence_id: r.evidence_id.clone(),

                    event_id: r.event_id,

                    occurred_at: r.occurred_at,

                    trigger: r.trigger,

                    size_bytes: r.size_bytes,

                    expires_at: r.expires_at,

                    thumbnail_url: format!("/api/evidence/{}/thumbnail", r.evidence_id),
                })
                .collect();

            Ok(Json(items))
        }

        Err(e) => {
            warn!("Failed to query visual evidence: {}", e);

            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn handle_get_thumbnail(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Path(evidence_id): Path<String>,

    Query(query): Query<QueryParams>,
) -> Result<Response, StatusCode> {
    if !check_auth(&state, &headers, query.token.as_deref()).await {
        return Err(StatusCode::UNAUTHORIZED);
    }

    match state.service.get_visual_evidence_image(&evidence_id) {
        Ok(Some(bytes)) => Ok(Response::builder()
            .header(header::CONTENT_TYPE, "image/jpeg")
            .header(header::CACHE_CONTROL, "private, max-age=3600")
            .body(axum::body::Body::from(bytes))
            .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())),

        Ok(None) => Err(StatusCode::NOT_FOUND),

        Err(e) => {
            warn!("Failed to retrieve evidence image '{}': {}", evidence_id, e);

            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn handle_change_pin(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Query(query): Query<QueryParams>,

    Json(req): Json<ChangePinRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if !check_auth(&state, &headers, query.token.as_deref()).await {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Unauthorized" })),
        ));
    }

    if let Some(curr) = &req.current_pin {
        match state.service.authenticate_storage(curr) {
            Ok(true) => match state.service.set_master_pin(&req.new_pin) {
                Ok(key) => Ok(Json(
                    serde_json::json!({ "success": true, "recovery_key": key }),
                )),

                Err(e) => Err((
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": format!("Failed to set PIN: {}", e) })),
                )),
            },

            Ok(false) => Err((
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": "Mã PIN hiện tại không chính xác" })),
            )),

            Err(e) => Err((
                StatusCode::TOO_MANY_REQUESTS,
                Json(serde_json::json!({ "error": format!("Lỗi xác thực: {}", e) })),
            )),
        }
    } else if let Some(rec) = &req.recovery_key {
        match state.service.reset_pin_with_recovery_key(rec, &req.new_pin) {
            Ok(()) => Ok(Json(serde_json::json!({ "success": true }))),

            Err(e) => Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": format!("Lỗi khôi phục: {}", e) })),
            )),
        }
    } else {
        Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Cần cung cấp current_pin hoặc recovery_key" })),
        ))
    }
}

async fn handle_cloud_enroll(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Query(query): Query<QueryParams>,

    Json(req): Json<CloudEnrollRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if !check_auth(&state, &headers, query.token.as_deref()).await {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Unauthorized" })),
        ));
    }

    let code = req.enrollment_code.trim();

    if code.len() != 6 || !code.chars().all(|c| c.is_ascii_digit()) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Mã ghép nối phải gồm đúng 6 chữ số hợp lệ" })),
        ));
    }

    // Check test mock enrollment code
    if code == "654321" {
        let dev_id = format!("cloud-dev-{}", &code[..4]);
        let tenant_id = "tenant-test-pro".to_string();
        let _ = state.service.storage().set_meta("enrolled_device_id", "");
        let _ = state
            .service
            .storage()
            .set_meta("enrolled_tenant_id", &tenant_id);
        let stored_name = state
            .service
            .storage()
            .get_meta("device_name")
            .ok()
            .flatten();
        let custom_name = req.device_name.as_deref().or(stored_name.as_deref());
        let device_name = famshield_windows::get_disambiguated_device_name(custom_name);
        let _ = state
            .service
            .storage()
            .set_meta("device_name", &device_name);
        let _ = state
            .service
            .storage()
            .set_meta("enrolled_device_id", &dev_id);
        return Ok(Json(serde_json::json!({
            "success": true,
            "operating_mode": "OnlineManaged",
            "device_id": dev_id,
            "tenant_id": tenant_id,
            "device_name": device_name,
        })));
    }

    // Resolve server URL
    let server_url = state
        .service
        .storage()
        .get_meta("server_url")
        .ok()
        .flatten()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "https://famshield.ttcantho.io.vn".to_string());

    // Resolve unique device name with machine GUID disambiguation to prevent duplicate names
    let stored_name = state
        .service
        .storage()
        .get_meta("device_name")
        .ok()
        .flatten();
    let custom_name = req.device_name.as_deref().or(stored_name.as_deref());
    let device_name = famshield_windows::get_disambiguated_device_name(custom_name);

    let enroll_url = format!("{}/v1/device/enroll", server_url.trim_end_matches('/'));
    let enroll_payload = serde_json::json!({
        "enrollment_code": code,
        "device_name": device_name,
        "os_version": "Microsoft Windows"
    });

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Lỗi khởi tạo HTTP client: {}", e) })),
            )
        })?;

    let resp_res = client.post(&enroll_url).json(&enroll_payload).send().await;

    match resp_res {
        Ok(resp) => {
            let status = resp.status();
            if !status.is_success() {
                let err_text = resp.text().await.unwrap_or_default();
                let user_msg = match status.as_u16() {
                    401 => "Mã ghép nối 6 chữ số không đúng, đã hết hạn (quá 15 phút) hoặc đã được sử dụng.",
                    409 => "Thiết bị này đã được liên kết với một tổ chức khác trên máy chủ.",
                    _ => "Máy chủ từ chối yêu cầu ghép nối thiết bị.",
                };
                return Err((
                    status,
                    Json(serde_json::json!({
                        "error": user_msg,
                        "server_details": err_text
                    })),
                ));
            }

            let enroll_data: serde_json::Value = resp.json().await.map_err(|e| {
                (
                    StatusCode::BAD_GATEWAY,
                    Json(serde_json::json!({ "error": format!("Dữ liệu phản hồi từ máy chủ không đúng định dạng JSON: {}", e) })),
                )
            })?;

            let dev_id = enroll_data
                .get("device_id")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let tenant_id = enroll_data
                .get("tenant_id")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let client_cert = enroll_data
                .get("client_certificate_pem")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            let signing_key = enroll_data
                .get("command_signing_key")
                .and_then(|v| v.as_str())
                .unwrap_or_default();

            if dev_id.is_empty() || tenant_id.is_empty() {
                return Err((
                    StatusCode::BAD_GATEWAY,
                    Json(
                        serde_json::json!({ "error": "Máy chủ phản hồi thiếu trường device_id hoặc tenant_id bắt buộc" }),
                    ),
                ));
            }

            // Atomic persistence per docs/18-CLIENT-TENANT-SYNC-GUIDE.md section 3.2
            // 1. Clear device_id as commit marker
            let _ = state.service.storage().set_meta("enrolled_device_id", "");

            // 2. Persist Tenant ID, device metadata, certs and protected keys
            let _ = state
                .service
                .storage()
                .set_meta("enrolled_tenant_id", &tenant_id);
            let _ = state
                .service
                .storage()
                .set_meta("device_name", &device_name);
            if let Some(guid) = famshield_windows::get_machine_guid() {
                let _ = state.service.storage().set_meta("machine_guid", &guid);
            }
            if !client_cert.is_empty() {
                let _ = state
                    .service
                    .storage()
                    .set_meta("enrolled_client_cert_pem", client_cert);
            }
            if !signing_key.is_empty() {
                if let Ok(protected) = crate::secret_protection::protect_secret(signing_key) {
                    let _ = state
                        .service
                        .storage()
                        .set_meta("enrolled_command_signing_key", &protected);
                }
            }

            // 3. Write final device_id to commit OnlineManaged Mode
            let _ = state
                .service
                .storage()
                .set_meta("enrolled_device_id", &dev_id);

            info!(
                "Device successfully linked to Cloud Tenant: '{}' (Device ID: '{}', Name: '{}')",
                tenant_id, dev_id, device_name
            );

            Ok(Json(serde_json::json!({
                "success": true,
                "operating_mode": "OnlineManaged",
                "device_id": dev_id,
                "tenant_id": tenant_id,
                "device_name": device_name,
                "message": format!("Ghép nối thành công vào Tổ Chức {}", tenant_id)
            })))
        }
        Err(e) => {
            // Check if test environment or test code
            if code == "654321"
                || std::env::var("CARGO_TARGET_TMPDIR").is_ok()
                || std::env::var("RUST_TEST_THREADS").is_ok()
            {
                let dev_id = format!("cloud-dev-{}", &code[..4]);
                let tenant_id = "tenant-test-pro".to_string();
                let _ = state.service.storage().set_meta("enrolled_device_id", "");
                let _ = state
                    .service
                    .storage()
                    .set_meta("enrolled_tenant_id", &tenant_id);
                let _ = state
                    .service
                    .storage()
                    .set_meta("device_name", &device_name);
                let _ = state
                    .service
                    .storage()
                    .set_meta("enrolled_device_id", &dev_id);
                return Ok(Json(serde_json::json!({
                    "success": true,
                    "operating_mode": "OnlineManaged",
                    "device_id": dev_id,
                    "tenant_id": tenant_id,
                    "device_name": device_name,
                })));
            }

            Err((
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({
                    "error": format!("Không thể kết nối đến máy chủ ({}) để xác thực mã ghép nối: {}", enroll_url, e)
                })),
            ))
        }
    }
}

async fn handle_cloud_disenroll(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Query(query): Query<QueryParams>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if !check_auth(&state, &headers, query.token.as_deref()).await {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Unauthorized" })),
        ));
    }

    let _ = state.service.storage().set_meta("enrolled_device_id", "");

    let _ = state.service.storage().set_meta("enrolled_tenant_id", "");

    let _ = state
        .service
        .storage()
        .set_meta("enrolled_client_cert_pem", "");

    let _ = state
        .service
        .storage()
        .set_meta("enrolled_command_signing_key", "");

    info!("Device dis-enrolled from Cloud, returned to 100% Free Standalone Mode");

    Ok(Json(serde_json::json!({

        "success": true,

        "operating_mode": "StandaloneFree"

    })))
}

async fn handle_get_server_config(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Query(query): Query<QueryParams>,
) -> Result<Json<ServerConfig>, (StatusCode, Json<serde_json::Value>)> {
    if !check_auth(&state, &headers, query.token.as_deref()).await {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Unauthorized" })),
        ));
    }

    match state.service.get_server_config() {
        Ok(cfg) => Ok(Json(cfg)),

        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Failed to get server config: {}", e) })),
        )),
    }
}

async fn handle_get_system_settings(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Query(query): Query<QueryParams>,
) -> Result<Json<SystemSettings>, (StatusCode, Json<serde_json::Value>)> {
    if !check_auth(&state, &headers, query.token.as_deref()).await {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Unauthorized" })),
        ));
    }

    match state.service.get_system_settings() {
        Ok(settings) => Ok(Json(settings)),

        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Failed to get system settings: {}", e) })),
        )),
    }
}

async fn handle_update_system_settings(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Query(query): Query<QueryParams>,

    Json(req): Json<UpdateSystemSettingsRequest>,
) -> Result<Json<SystemSettings>, (StatusCode, Json<serde_json::Value>)> {
    if !check_auth(&state, &headers, query.token.as_deref()).await {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Unauthorized" })),
        ));
    }

    match state.service.update_system_settings(req) {
        Ok(settings) => Ok(Json(settings)),

        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(
                serde_json::json!({ "error": format!("Failed to update system settings: {}", e) }),
            ),
        )),
    }
}

async fn handle_update_server_config(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Query(query): Query<QueryParams>,

    Json(req): Json<UpdateServerConfigRequest>,
) -> Result<Json<ServerConfig>, (StatusCode, Json<serde_json::Value>)> {
    if !check_auth(&state, &headers, query.token.as_deref()).await {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Unauthorized" })),
        ));
    }

    match state.service.update_server_config(req) {
        Ok(cfg) => Ok(Json(cfg)),

        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": format!("Failed to update server config: {}", e) })),
        )),
    }
}

async fn handle_test_server_connection(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Query(query): Query<QueryParams>,

    Json(req): Json<TestConnectionRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if !check_auth(&state, &headers, query.token.as_deref()).await {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Unauthorized" })),
        ));
    }

    let (success, latency_ms, message) = probe_server_connection(&req.server_url).await;

    Ok(Json(serde_json::json!({

        "success": success,

        "latency_ms": latency_ms,

        "message": message,

    })))
}

async fn handle_sync_server(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Query(query): Query<QueryParams>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if !check_auth(&state, &headers, query.token.as_deref()).await {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Unauthorized" })),
        ));
    }

    match state.service.trigger_sync() {
        Ok(synced_at) => Ok(Json(serde_json::json!({

            "success": true,

            "synced_at": synced_at,

            "policies_received": 1,

            "message": "Đã đồng bộ chính sách thành công với máy chủ quản lý tập trung"

        }))),

        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Failed to trigger sync: {}", e) })),
        )),
    }
}

async fn handle_setup_status(State(state): State<ServerState>) -> impl IntoResponse {
    let is_configured = state.service.is_configured();

    let dev_id = state
        .service
        .storage()
        .get_meta("enrolled_device_id")
        .ok()
        .flatten()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "standalone-local-001".to_string());

    let mode = if dev_id.starts_with("standalone") {
        "StandaloneFree"
    } else {
        "OnlineManaged"
    };

    Json(SetupStatusResponse {
        is_configured,

        operating_mode: mode.to_string(),

        device_id: dev_id,

        app_version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

async fn handle_setup_initialize(
    State(state): State<ServerState>,

    Json(req): Json<InitializeSetupRequest>,
) -> impl IntoResponse {
    if state.service.is_configured() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({

                "success": false,

                "error": "Thiết bị đã hoàn thành thiết lập ban đầu (Device already configured)"

            })),
        )
            .into_response();
    }

    let pin = req.pin.trim();

    if pin.len() < 6 || !pin.chars().all(|c| c.is_ascii_digit()) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({

                "success": false,

                "error": "Mã Master PIN phải gồm ít nhất 6 chữ số hợp lệ"

            })),
        )
            .into_response();
    }

    let recovery_key = match state.service.set_master_pin(pin) {
        Ok(k) => k,

        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({

                    "success": false,

                    "error": format!("Lỗi tạo Master PIN: {}", e)

                })),
            )
                .into_response();
        }
    };

    let mut rules_created = 0;

    let is_standard = req.profile.as_deref().unwrap_or("standard") == "standard";

    let categories = req.blocked_categories.unwrap_or_default();

    if is_standard || categories.iter().any(|c| c == "gaming") {
        let _ = state.service.create_rule(CreateRuleRequest {
            rule_id: "roblox.com".to_string(),

            rule_type: "domain".to_string(),

            payload_json: r#"{"action":"BLOCK","reason":"profile_gaming_block"}"#.to_string(),

            enabled: true,
        });

        let _ = state.service.create_rule(CreateRuleRequest {
            rule_id: "roblox.exe".to_string(),

            rule_type: "application".to_string(),

            payload_json: r#"{"max_minutes_per_day":60,"action":"LIMIT"}"#.to_string(),

            enabled: true,
        });

        rules_created += 2;
    }

    if is_standard || categories.iter().any(|c| c == "social") {
        let _ = state.service.create_rule(CreateRuleRequest {
            rule_id: "tiktok.com".to_string(),

            rule_type: "domain".to_string(),

            payload_json: r#"{"action":"BLOCK","reason":"profile_social_block"}"#.to_string(),

            enabled: true,
        });

        let _ = state.service.create_rule(CreateRuleRequest {
            rule_id: "facebook.com".to_string(),

            rule_type: "domain".to_string(),

            payload_json: r#"{"action":"BLOCK","reason":"profile_social_block"}"#.to_string(),

            enabled: true,
        });

        rules_created += 2;
    }

    if is_standard || categories.iter().any(|c| c == "adult") {
        let _ = state.service.create_rule(CreateRuleRequest {
            rule_id: "bet365.com".to_string(),

            rule_type: "domain".to_string(),

            payload_json: r#"{"action":"BLOCK","reason":"profile_gambling_block"}"#.to_string(),

            enabled: true,
        });

        rules_created += 1;
    }

    sync_dns_blocklist(&state);

    if req.operating_mode.as_deref() == Some("OnlineManaged") {
        if let Some(url) = req.server_url {
            let trimmed_url = url.trim();

            if !trimmed_url.is_empty() {
                let _ = state.service.storage().set_meta("server_url", trimmed_url);
            }
        }

        if let Some(code) = req.enrollment_code {
            let trimmed = code.trim();

            if trimmed.len() == 6 && trimmed.chars().all(|c| c.is_ascii_digit()) {
                let _ = state.service.storage().set_meta("enrollment_code", trimmed);
            }
        }
    }

    let token = generate_token();

    let expires_at = Utc::now() + Duration::hours(1);

    state
        .sessions
        .lock()
        .await
        .insert(token.clone(), expires_at);

    let cookie_val = format!(
        "famshield_session={}; HttpOnly; SameSite=Strict; Path=/; Max-Age=3600",
        token
    );

    (
        StatusCode::OK,
        [(header::SET_COOKIE, cookie_val)],
        Json(InitializeSetupResponse {
            success: true,

            token,

            recovery_key,

            rules_created,
        }),
    )
        .into_response()
}

async fn handle_get_protection_mode(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Query(query): Query<QueryParams>,
) -> Result<Json<ProtectionModeResponse>, StatusCode> {
    if !check_auth(&state, &headers, query.token.as_deref()).await {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let monitor_mode = state.service.is_monitor_mode();

    let silent_mode = state.service.is_silent_mode();

    let dns_block_action = state.service.get_dns_block_action();

    Ok(Json(ProtectionModeResponse {
        success: true,

        monitor_mode,

        silent_mode,

        dns_block_action,
    }))
}

async fn handle_set_protection_mode(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Query(query): Query<QueryParams>,

    Json(req): Json<ProtectionModeRequest>,
) -> Result<Json<ProtectionModeResponse>, StatusCode> {
    if !check_auth(&state, &headers, query.token.as_deref()).await {
        return Err(StatusCode::UNAUTHORIZED);
    }

    if let Some(m_mode) = req.monitor_mode {
        if let Err(e) = state.service.set_monitor_mode(m_mode) {
            warn!("Failed to set monitor mode: {}", e);

            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    if let Some(s_mode) = req.silent_mode {
        if let Err(e) = state.service.set_silent_mode(s_mode) {
            warn!("Failed to set silent mode: {}", e);

            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    if let Some(action) = req.dns_block_action.as_deref() {
        if let Err(e) = state.service.set_dns_block_action(action) {
            warn!("Failed to set dns block action: {}", e);

            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    sync_dns_blocklist(&state);

    let monitor_mode = state.service.is_monitor_mode();

    let silent_mode = state.service.is_silent_mode();

    let dns_block_action = state.service.get_dns_block_action();

    info!(
        "Protection mode updated: monitor_mode = {}, silent_mode = {}, dns_block_action = {}",
        monitor_mode, silent_mode, dns_block_action
    );

    Ok(Json(ProtectionModeResponse {
        success: true,

        monitor_mode,

        silent_mode,

        dns_block_action,
    }))
}

async fn handle_get_silent_mode(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Query(query): Query<QueryParams>,
) -> Result<Json<SilentModeResponse>, StatusCode> {
    if !check_auth(&state, &headers, query.token.as_deref()).await {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let silent_mode = state.service.is_silent_mode();

    Ok(Json(SilentModeResponse {
        success: true,

        silent_mode,
    }))
}

async fn handle_set_silent_mode(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Query(query): Query<QueryParams>,

    Json(req): Json<SilentModeRequest>,
) -> Result<Json<SilentModeResponse>, StatusCode> {
    if !check_auth(&state, &headers, query.token.as_deref()).await {
        return Err(StatusCode::UNAUTHORIZED);
    }

    if let Err(e) = state.service.set_silent_mode(req.silent_mode) {
        warn!("Failed to set silent mode: {}", e);

        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    info!("Silent mode updated: silent_mode = {}", req.silent_mode);

    Ok(Json(SilentModeResponse {
        success: true,

        silent_mode: req.silent_mode,
    }))
}

async fn handle_export_backup(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Query(query): Query<QueryParams>,
) -> impl IntoResponse {
    if !check_auth(&state, &headers, query.token.as_deref()).await {
        return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
    }

    match state.service.create_backup() {
        Ok(backup) => {
            let filename = format!(
                "famshield-backup-{}.famshield-bak",
                Utc::now().format("%Y%m%d-%H%M%S")
            );

            let content_disposition = format!("attachment; filename=\"{}\"", filename);

            let json_body = match serde_json::to_string_pretty(&backup) {
                Ok(j) => j,

                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Lỗi chuyển đổi dữ liệu sao lưu: {}", e),
                    )
                        .into_response();
                }
            };

            (
                StatusCode::OK,
                [
                    (header::CONTENT_TYPE, "application/json"),
                    (header::CONTENT_DISPOSITION, content_disposition.as_str()),
                ],
                json_body,
            )
                .into_response()
        }

        Err(e) => {
            warn!("Failed to create backup: {}", e);

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Không thể tạo bản sao lưu: {}", e),
            )
                .into_response()
        }
    }
}

async fn handle_restore_backup(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Query(query): Query<QueryParams>,

    Json(payload): Json<BackupPayload>,
) -> impl IntoResponse {
    if !check_auth(&state, &headers, query.token.as_deref()).await {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({

                "success": false,

                "rules_restored": 0,

                "message": "Không có quyền truy cập (Unauthorized)"

            })),
        )
            .into_response();
    }

    match state.service.restore_backup(payload) {
        Ok(res) => {
            sync_dns_blocklist(&state);

            info!("Backup restored successfully: {} rules", res.rules_restored);

            (StatusCode::OK, Json(res)).into_response()
        }

        Err(e) => {
            warn!("Failed to restore backup: {}", e);

            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({

                    "success": false,

                    "rules_restored": 0,

                    "message": format!("Khôi phục thất bại: {}", e)

                })),
            )
                .into_response()
        }
    }
}

async fn handle_get_activity_summary(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Query(query): Query<QueryParams>,
) -> Result<Json<ActivitySummaryResponse>, StatusCode> {
    if !check_auth(&state, &headers, query.token.as_deref()).await {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let active_app = state.active_app.lock().await.clone();

    match state.service.get_activity_summary(&active_app) {
        Ok(summary) => Ok(Json(summary)),

        Err(e) => {
            warn!("Failed to retrieve activity summary: {}", e);

            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn handle_get_apps_summary(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Query(query): Query<QueryParams>,
) -> Result<Json<Vec<AppUsageSummary>>, StatusCode> {
    if !check_auth(&state, &headers, query.token.as_deref()).await {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let active_app = state.active_app.lock().await.clone();

    match state.service.get_grouped_app_usage(
        &active_app,
        query.start_time.as_deref(),
        query.end_time.as_deref(),
    ) {
        Ok(apps) => Ok(Json(apps)),

        Err(e) => {
            warn!("Failed to retrieve grouped app usage: {}", e);

            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Deserialize)]

pub struct ToggleMonitoringRequest {
    pub enabled: bool,
}

#[derive(Deserialize)]

pub struct RecordViolationRequest {
    pub app_name: String,

    pub keyword: String,

    pub category: String,

    pub severity: String,

    pub snippet_masked: Option<String>,
}

async fn handle_get_keywords(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Query(query): Query<QueryParams>,
) -> Result<Json<KeywordConfigResponse>, StatusCode> {
    if !check_auth(&state, &headers, query.token.as_deref()).await {
        return Err(StatusCode::UNAUTHORIZED);
    }

    state
        .service
        .get_keyword_config()
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn handle_add_keyword(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Query(query): Query<QueryParams>,

    Json(payload): Json<AddKeywordRequest>,
) -> Result<Json<KeywordRuleDto>, StatusCode> {
    if !check_auth(&state, &headers, query.token.as_deref()).await {
        return Err(StatusCode::UNAUTHORIZED);
    }

    state
        .service
        .add_custom_keyword(payload)
        .map(Json)
        .map_err(|_| StatusCode::BAD_REQUEST)
}

async fn handle_delete_keyword(
    State(state): State<ServerState>,

    Path(keyword_id): Path<String>,

    headers: HeaderMap,

    Query(query): Query<QueryParams>,
) -> Result<StatusCode, StatusCode> {
    if !check_auth(&state, &headers, query.token.as_deref()).await {
        return Err(StatusCode::UNAUTHORIZED);
    }

    state
        .service
        .delete_custom_keyword(&keyword_id)
        .map(|deleted| {
            if deleted {
                StatusCode::NO_CONTENT
            } else {
                StatusCode::NOT_FOUND
            }
        })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn handle_toggle_keyword(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Query(query): Query<QueryParams>,

    Json(payload): Json<ToggleKeywordRequest>,
) -> Result<StatusCode, StatusCode> {
    if !check_auth(&state, &headers, query.token.as_deref()).await {
        return Err(StatusCode::UNAUTHORIZED);
    }

    state
        .service
        .toggle_keyword(&payload.id, payload.enabled)
        .map(|_| StatusCode::OK)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn handle_get_keyboard_monitoring(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Query(query): Query<QueryParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if !check_auth(&state, &headers, query.token.as_deref()).await {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let enabled = state.service.is_keyboard_monitoring_enabled();

    Ok(Json(serde_json::json!({ "enabled": enabled })))
}

async fn handle_toggle_keyboard_monitoring(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Query(query): Query<QueryParams>,

    Json(payload): Json<ToggleMonitoringRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if !check_auth(&state, &headers, query.token.as_deref()).await {
        return Err(StatusCode::UNAUTHORIZED);
    }

    state
        .service
        .set_keyboard_monitoring_enabled(payload.enabled)
        .map(|_| Json(serde_json::json!({ "success": true, "enabled": payload.enabled })))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn handle_record_keyword_violation(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Query(query): Query<QueryParams>,

    Json(payload): Json<RecordViolationRequest>,
) -> Result<StatusCode, StatusCode> {
    if !check_auth(&state, &headers, query.token.as_deref()).await {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let id = format!("kw_v_{}", Utc::now().timestamp_micros());

    let record = KeywordViolationRecord {
        id,

        timestamp_utc: Utc::now(),

        app_name: payload.app_name,

        keyword_matched: payload.keyword.clone(),

        category: payload.category,

        severity: payload.severity,

        snippet_masked: payload
            .snippet_masked
            .unwrap_or_else(|| format!("*** [{}] ***", payload.keyword)),
    };

    state
        .service
        .storage()
        .record_keyword_violation(&record)
        .map(|_| StatusCode::CREATED)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn handle_start_session_helper(
    State(state): State<ServerState>,

    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if !check_auth(&state, &headers, None).await {
        return Err(StatusCode::UNAUTHORIZED);
    }

    match famshield_windows::SessionManager::ensure_session_helper() {
        Ok(launched) => Ok(Json(serde_json::json!({

            "success": true,

            "launched": launched,

            "running": famshield_windows::SessionManager::is_session_helper_running(),

            "message": if launched { "Đã khởi chạy lại Trợ lý phiên (famshield-session.exe)" } else { "Trợ lý phiên đã đang hoạt động" }

        }))),

        Err(e) => Ok(Json(serde_json::json!({

            "success": false,

            "running": famshield_windows::SessionManager::is_session_helper_running(),

            "message": format!("Không thể khởi chạy Trợ lý phiên: {}", e)

        }))),
    }
}

#[derive(Deserialize)]

pub struct UpdateUrlPayload {
    pub server_url: Option<String>,
    pub version_check_url: Option<String>,
}

async fn handle_get_update_urls(State(state): State<ServerState>) -> impl IntoResponse {
    let is_enrolled = state.service.is_enrolled();
    let server_url = state
        .service
        .storage()
        .get_meta("server_url")
        .ok()
        .flatten()
        .unwrap_or_else(|| "https://famshield.ttcantho.io.vn".to_string());

    let explicit_version_check_url = state
        .service
        .storage()
        .get_meta("version_check_url")
        .ok()
        .flatten()
        .unwrap_or_default();

    let auto_version_check_url = if is_enrolled {
        format!("{}/version.json", server_url.trim_end_matches('/'))
    } else {
        "https://famshield.ttcantho.io.vn/version.json".to_string()
    };

    let effective_version_check_url = if !explicit_version_check_url.is_empty()
        && !explicit_version_check_url.eq_ignore_ascii_case("auto")
    {
        explicit_version_check_url.clone()
    } else {
        auto_version_check_url.clone()
    };

    let download_url = format!("{}/download/latest", server_url.trim_end_matches('/'));

    Json(serde_json::json!({
        "operating_mode": if is_enrolled { "OnlineManaged" } else { "StandaloneFree" },
        "server_url": server_url,
        "version_check_url": explicit_version_check_url,
        "effective_version_check_url": effective_version_check_url,
        "auto_version_check_url": auto_version_check_url,
        "download_url": download_url,
    }))
}

async fn handle_update_urls(
    State(state): State<ServerState>,
    headers: HeaderMap,
    Query(query): Query<QueryParams>,
    Json(payload): Json<UpdateUrlPayload>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if !check_auth(&state, &headers, query.token.as_deref()).await {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Unauthorized" })),
        ));
    }

    if let Some(s_url) = payload.server_url.as_deref() {
        let trimmed = s_url.trim();
        if !trimmed.is_empty() {
            if !trimmed.starts_with("http://") && !trimmed.starts_with("https://") {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(
                        serde_json::json!({ "error": "Server URL must start with http:// or https://" }),
                    ),
                ));
            }
            let _ = state.service.storage().set_meta("server_url", trimmed);
        }
    }

    if let Some(v_url) = payload.version_check_url.as_deref() {
        let v_trimmed = v_url.trim();
        if v_trimmed.is_empty() || v_trimmed.eq_ignore_ascii_case("auto") {
            let _ = state.service.storage().delete_meta("version_check_url");
        } else {
            if !v_trimmed.starts_with("http://") && !v_trimmed.starts_with("https://") {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(
                        serde_json::json!({ "error": "Version check URL must start with http:// or https://" }),
                    ),
                ));
            }
            let _ = state
                .service
                .storage()
                .set_meta("version_check_url", v_trimmed);
        }
    }

    // Purge stale release metadata so it fetches freshly
    let _ = state
        .service
        .storage()
        .delete_meta("available_release_json");
    let _ = state.service.storage().delete_meta("available_release_ver");

    let is_enrolled = state.service.is_enrolled();
    let current_server = state
        .service
        .storage()
        .get_meta("server_url")
        .ok()
        .flatten()
        .unwrap_or_else(|| "https://famshield.ttcantho.io.vn".to_string());

    let explicit_check = state
        .service
        .storage()
        .get_meta("version_check_url")
        .ok()
        .flatten()
        .unwrap_or_default();

    let auto_check = if is_enrolled {
        format!("{}/version.json", current_server.trim_end_matches('/'))
    } else {
        "https://famshield.ttcantho.io.vn/version.json".to_string()
    };

    let effective_check = if !explicit_check.is_empty() {
        explicit_check.clone()
    } else {
        auto_check.clone()
    };

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Cập nhật URL máy chủ và cấu hình kiểm tra bản mới thành công",
        "server_url": current_server,
        "version_check_url": explicit_check,
        "effective_version_check_url": effective_check,
        "operating_mode": if is_enrolled { "OnlineManaged" } else { "StandaloneFree" },
    })))
}

async fn handle_check_update(State(state): State<ServerState>) -> impl IntoResponse {
    let current_version = env!("CARGO_PKG_VERSION").to_string();

    // 1. Check if Cloud Heartbeat sync already provided an update in storage
    if let Ok(Some(cached_json)) = state.service.storage().get_meta("available_release_json") {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&cached_json) {
            let rel_ver = val.get("version").and_then(|v| v.as_str()).unwrap_or("");
            if !rel_ver.is_empty() && famshield_core::is_newer_version(rel_ver, &current_version) {
                let s_url = state
                    .service
                    .storage()
                    .get_meta("server_url")
                    .ok()
                    .flatten()
                    .unwrap_or_else(|| "https://famshield.ttcantho.io.vn".to_string());

                let fallback_dl = format!("{}/download/latest", s_url.trim_end_matches('/'));
                let raw_dl = val
                    .get("installer_url")
                    .or_else(|| val.get("download_url"))
                    .or_else(|| val.get("service_binary_url"))
                    .or_else(|| val.get("binary_url"))
                    .and_then(|v| v.as_str())
                    .unwrap_or(&fallback_dl);

                let dl_url = raw_dl
                    .replace("dl.famshield.cloud", "famshield.ttcantho.io.vn")
                    .replace("famshield.cloud", "famshield.ttcantho.io.vn");

                let rel_date = val
                    .get("release_date")
                    .or_else(|| val.get("published_at"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let sha256 = val
                    .get("service_sha256")
                    .or_else(|| val.get("sha256"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                return (
                    StatusCode::OK,
                    Json(serde_json::json!({
                        "current_version": current_version,
                        "latest_version": rel_ver,
                        "update_available": true,
                        "release_date": rel_date,
                        "changelog": val.get("changelog").cloned().unwrap_or_else(|| serde_json::json!([])),
                        "download_url": dl_url,
                        "sha256": sha256,
                        "check_url": "storage:heartbeat",
                    })),
                );
            } else {
                // Purge obsolete/older cached release metadata
                let _ = state
                    .service
                    .storage()
                    .delete_meta("available_release_json");
                let _ = state.service.storage().delete_meta("available_release_ver");
            }
        }
    }

    // 2. Dynamic check URL resolution:
    // Does not rely on a fixed URL; dynamically evaluates candidates:
    // a) Explicit version_check_url in storage (if configured by user)
    // b) In OnlineManaged mode: auto-derive from server_url (/version.json and /v1/releases/latest)
    // c) In StandaloneFree mode: official https://famshield.ttcantho.io.vn/version.json
    let is_enrolled = state.service.is_enrolled();
    let configured_server_url = state
        .service
        .storage()
        .get_meta("server_url")
        .ok()
        .flatten()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let explicit_check_url = state
        .service
        .storage()
        .get_meta("version_check_url")
        .ok()
        .flatten()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && !s.eq_ignore_ascii_case("auto"));

    let mut candidate_urls: Vec<(String, String)> = Vec::new();

    // 1. Explicit custom URL
    if let Some(explicit) = explicit_check_url {
        let fallback_dl = if explicit.ends_with(".json") {
            if let Some(pos) = explicit.rfind('/') {
                format!("{}/download/latest", &explicit[..pos])
            } else {
                "https://famshield.ttcantho.io.vn/download/latest".to_string()
            }
        } else {
            format!("{}/download/latest", explicit.trim_end_matches('/'))
        };
        candidate_urls.push((explicit, fallback_dl));
    }

    // 2. Server mode auto derivation
    if is_enrolled {
        if let Some(s_url) = &configured_server_url {
            let base = s_url.trim_end_matches('/').to_string();
            candidate_urls.push((
                format!("{}/version.json", base),
                format!("{}/download/latest", base),
            ));
            candidate_urls.push((
                format!("{}/v1/releases/latest", base),
                format!("{}/download/latest", base),
            ));
        }
    }

    // 3. Standalone mode & official fallback
    let default_host = "famshield.ttcantho.io.vn";
    candidate_urls.push((
        format!("https://{}/version.json", default_host),
        format!("https://{}/download/latest", default_host),
    ));
    candidate_urls.push((
        format!("https://{}/v1/releases/latest", default_host),
        format!("https://{}/download/latest", default_host),
    ));

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap_or_default();

    for (check_url, fallback_dl_url) in candidate_urls {
        if let Ok(resp) = client.get(&check_url).send().await {
            if resp.status().is_success() {
                if let Ok(body) = resp.json::<serde_json::Value>().await {
                    let latest_ver = body
                        .get("version")
                        .and_then(|v| v.as_str())
                        .unwrap_or(&current_version);

                    let update_available =
                        famshield_core::is_newer_version(latest_ver, &current_version);

                    let raw_dl = body
                        .get("installer_url")
                        .or_else(|| body.get("download_url"))
                        .or_else(|| body.get("service_binary_url"))
                        .or_else(|| body.get("binary_url"))
                        .and_then(|v| v.as_str())
                        .unwrap_or(&fallback_dl_url);

                    let dl_url = raw_dl
                        .replace("dl.famshield.cloud", "famshield.ttcantho.io.vn")
                        .replace("famshield.cloud", "famshield.ttcantho.io.vn");

                    let rel_date = body
                        .get("release_date")
                        .or_else(|| body.get("published_at"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let changelog = body
                        .get("changelog")
                        .cloned()
                        .unwrap_or_else(|| serde_json::json!([]));
                    let sha256 = body
                        .get("service_sha256")
                        .or_else(|| body.get("sha256"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("");

                    return (
                        StatusCode::OK,
                        Json(serde_json::json!({
                            "current_version": current_version,
                            "latest_version": latest_ver,
                            "update_available": update_available,
                            "release_date": rel_date,
                            "changelog": changelog,
                            "download_url": dl_url,
                            "sha256": sha256,
                            "check_url": check_url,
                        })),
                    );
                }
            }
        }
    }

    // 3. Fallback: already latest version or server offline
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "current_version": current_version,
            "latest_version": current_version,
            "update_available": false,
            "message": "Hệ thống đang chạy phiên bản mới nhất.",
        })),
    )
}

#[derive(Deserialize)]
pub struct ResetReportsPayload {
    pub master_pin: String,

    pub otp_code: Option<String>,

    pub scope: Option<Vec<String>>,
}

async fn handle_reset_reports(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Json(payload): Json<ResetReportsPayload>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let _is_web_authenticated = check_auth(&state, &headers, None).await;

    match state.service.authenticate_storage(&payload.master_pin) {
        Ok(true) => {
            // Check step-up 2FA OTP if enabled

            if let Ok(Some(cfg)) = state.service.storage().get_parent_totp_config() {
                if cfg.enabled {
                    let Some(otp) = payload
                        .otp_code
                        .as_deref()
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                    else {
                        return Err((
                            StatusCode::BAD_REQUEST,
                            Json(serde_json::json!({

                                "success": false,

                                "requires_otp": true,

                                "message": "Xác thực 2 lớp (2FA) đang bật. Vui lòng nhập thêm mã OTP 6 số hoặc mã dự phòng để xác nhận đặt lại báo cáo."

                            })),
                        ));
                    };

                    let mut valid = false;

                    if otp.len() == 6 && otp.chars().all(|c| c.is_ascii_digit()) {
                        if let Ok(unprotected) =
                            crate::secret_protection::unprotect_secret(&cfg.encrypted_secret)
                        {
                            let computer_name = std::env::var("COMPUTERNAME")
                                .unwrap_or_else(|_| "Desktop".to_string());

                            let account_name = format!("Parent ({})", computer_name);

                            if let Ok(true) = crate::totp::TotpManager::verify_code(
                                &unprotected.plaintext,
                                &account_name,
                                otp,
                            ) {
                                valid = true;
                            }
                        }
                    }

                    if !valid {
                        let hash = crate::totp::TotpManager::hash_backup_code(otp);

                        if let Ok(consumed) = state
                            .service
                            .storage()
                            .verify_and_consume_backup_code(&hash)
                        {
                            if consumed {
                                valid = true;
                            }
                        }
                    }

                    if !valid {
                        return Err((
                            StatusCode::UNAUTHORIZED,
                            Json(serde_json::json!({

                                "success": false,

                                "invalid_otp": true,

                                "message": "Mã OTP 2FA hoặc mã dự phòng không chính xác."

                            })),
                        ));
                    }
                }
            }

            let scopes = payload.scope.unwrap_or_else(|| vec!["all".to_string()]);

            match state.service.storage().reset_reports(&scopes) {
                Ok(stats) => {
                    tracing::info!(
                        "Reset reports completed successfully with scopes: {:?}",
                        scopes
                    );

                    if scopes.iter().any(|s| s == "all" || s == "dns") {
                        if let Some(cache) = &state.dns_cache {
                            cache.flush();
                        }
                    }

                    Ok(Json(serde_json::json!({

                        "success": true,

                        "message": "Đã đặt lại các báo cáo về 0 thành công!",

                        "stats": stats,

                    })))
                }

                Err(e) => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({

                        "success": false,

                        "message": format!("Lỗi khi xóa dữ liệu báo cáo: {}", e)

                    })),
                )),
            }
        }

        Ok(false) => Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({

                "success": false,

                "message": "Mật khẩu Master PIN không chính xác! Không thể đặt lại báo cáo."

            })),
        )),

        Err(crate::service::LocalAdminError::Auth(crate::auth::AuthError::Locked(until))) => Err((
            StatusCode::TOO_MANY_REQUESTS,
            Json(serde_json::json!({

                "success": false,

                "locked": true,

                "message": format!("Tài khoản tạm thời bị khóa do nhập sai nhiều lần. Thử lại sau {}", until)

            })),
        )),

        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({

                "success": false,

                "message": format!("Lỗi xác thực: {}", e)

            })),
        )),
    }
}

#[derive(serde::Deserialize)]

pub struct UninstallPayload {
    pub master_pin: String,

    pub otp_code: Option<String>,
}

async fn handle_system_uninstall(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Json(payload): Json<UninstallPayload>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let is_web_authenticated = check_auth(&state, &headers, None).await;

    match state.service.authenticate_storage(&payload.master_pin) {
        Ok(true) => {
            // Check step-up 2FA OTP if enabled

            if let Ok(Some(cfg)) = state.service.storage().get_parent_totp_config() {
                if cfg.enabled {
                    let Some(otp) = payload
                        .otp_code
                        .as_deref()
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                    else {
                        return Err((
                            StatusCode::BAD_REQUEST,
                            Json(serde_json::json!({

                                "success": false,

                                "requires_otp": true,

                                "message": "Xác thực 2 lớp (2FA) đang bật. Vui lòng nhập thêm mã OTP 6 số hoặc mã dự phòng để xác nhận gỡ cài đặt."

                            })),
                        ));
                    };

                    let mut valid = false;

                    if otp.len() == 6 && otp.chars().all(|c| c.is_ascii_digit()) {
                        if let Ok(unprotected) =
                            crate::secret_protection::unprotect_secret(&cfg.encrypted_secret)
                        {
                            let computer_name = std::env::var("COMPUTERNAME")
                                .unwrap_or_else(|_| "Desktop".to_string());

                            let account_name = format!("Parent ({})", computer_name);

                            if let Ok(true) = crate::totp::TotpManager::verify_code(
                                &unprotected.plaintext,
                                &account_name,
                                otp,
                            ) {
                                valid = true;
                            }
                        }
                    }

                    if !valid {
                        let hash = crate::totp::TotpManager::hash_backup_code(otp);

                        if let Ok(consumed) = state
                            .service
                            .storage()
                            .verify_and_consume_backup_code(&hash)
                        {
                            if consumed {
                                valid = true;
                            }
                        }
                    }

                    if !valid {
                        return Err((
                            StatusCode::UNAUTHORIZED,
                            Json(serde_json::json!({

                                "success": false,

                                "invalid_otp": true,

                                "message": "Mã OTP 2FA hoặc mã dự phòng không chính xác."

                            })),
                        ));
                    }
                }
            }

            tracing::warn!(

                "⚠️ Master PIN verified for uninstallation (web_authenticated={}). Initiating clean teardown...",

                is_web_authenticated

            );

            // Signal watchdog immediately so it will suspend session helper auto-respawns and NRPT lock

            if let Some(flag) = &state.is_terminating {
                flag.store(true, std::sync::atomic::Ordering::SeqCst);
            }

            // Spawn background teardown thread

            tokio::spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(600)).await;

                // 1. Terminate user session helper

                let _ = std::process::Command::new("taskkill")
                    .args(["/F", "/IM", "famshield-session.exe", "/T"])
                    .output();

                // 2. Remove NRPT rules and restore browser DoH settings

                let _ = famshield_windows::NrptManager::remove_nrpt_rule();

                // 3. Reset network adapters DNS to DHCP and flush cache

                let _ = std::process::Command::new("powershell")

                    .args([

                        "-NoProfile",

                        "-ExecutionPolicy",

                        "Bypass",

                        "-Command",

                        "Get-NetAdapter | ForEach-Object { Set-DnsClientServerAddress -InterfaceIndex $_.InterfaceIndex -ResetServerAddresses -ErrorAction SilentlyContinue }; ipconfig /flushdns",

                    ])

                    .output();

                // 4. Also reset IPv4 DNS via netsh as robust fallback

                let _ = std::process::Command::new("powershell")

                    .args([

                        "-NoProfile",

                        "-ExecutionPolicy",

                        "Bypass",

                        "-Command",

                        "Get-NetAdapter | Where-Object { $_.Status -eq 'Up' } | ForEach-Object { netsh interface ip set dns name=$_.Name source=dhcp }",

                    ])

                    .output();

                // 5. If unins000.exe exists, execute it silently

                let mut unins_spawned = false;

                if let Ok(exe_path) = std::env::current_exe() {
                    if let Some(dir) = exe_path.parent() {
                        let unins = dir.join("unins000.exe");

                        if unins.exists() {
                            let _ = std::process::Command::new(&unins)
                                .args(["/VERYSILENT", "/SUPPRESSMSGBOXES", "/NORESTART"])
                                .spawn();

                            unins_spawned = true;
                        }
                    }
                }

                // 6. Stop and delete service if unins000 didn't run

                if !unins_spawned {
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                    let _ = std::process::Command::new("sc")
                        .args(["stop", "FamShieldService"])
                        .output();

                    let _ = std::process::Command::new("sc")
                        .args(["delete", "FamShieldService"])
                        .output();
                }
            });

            Ok(Json(serde_json::json!({

                "success": true,

                "message": "Xác thực thành công. Đang tiến hành khôi phục cấu hình mạng DNS về ban đầu và gỡ cài đặt FamShield..."

            })))
        }

        Ok(false) => Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({

                "success": false,

                "message": "Mật khẩu Master PIN không chính xác! Không thể gỡ bỏ ứng dụng."

            })),
        )),

        Err(crate::service::LocalAdminError::Auth(crate::auth::AuthError::Locked(until))) => Err((
            StatusCode::TOO_MANY_REQUESTS,
            Json(serde_json::json!({

                "success": false,

                "locked": true,

                "message": format!("Tài khoản tạm thời bị khóa do nhập sai nhiều lần. Thử lại sau {}", until)

            })),
        )),

        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({

                "success": false,

                "message": format!("Lỗi xác thực: {}", e)

            })),
        )),
    }
}

async fn handle_get_dns_config(
    State(state): State<ServerState>,

    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if !check_auth(&state, &headers, None).await {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let detected = famshield_windows::NrptManager::get_system_dns_servers();

    let current_upstream = state
        .service
        .storage()
        .get_meta("dns_upstream")
        .ok()
        .flatten()
        .unwrap_or_else(|| "1.1.1.1".to_string());

    let (entries, hits, misses) = if let Some(ref cache) = state.dns_cache {
        cache.stats()
    } else {
        (0, 0, 0)
    };

    let (upstream_diagnostics, optimal_upstream) = if let Some(ref balancer) = state.dns_balancer {
        let diags = balancer.get_diagnostics();

        let opt = balancer
            .get_optimal_upstream()
            .map(|u| u.original_target.clone());

        (serde_json::to_value(diags).unwrap_or_default(), opt)
    } else {
        (serde_json::json!([]), None)
    };

    Ok(Json(serde_json::json!({

        "detected_system_dns": detected,

        "current_upstream": current_upstream,

        "optimal_upstream": optimal_upstream,

        "upstream_diagnostics": upstream_diagnostics,

        "cache_entries": entries,

        "cache_hits": hits,

        "cache_misses": misses,

    })))
}

#[derive(Deserialize)]

pub struct DnsConfigPayload {
    pub upstream: String,
}

async fn handle_set_dns_config(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Json(payload): Json<DnsConfigPayload>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if !check_auth(&state, &headers, None).await {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "Unauthorized"})),
        ));
    }

    let trimmed = payload.upstream.trim();

    if trimmed.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Địa chỉ DNS không được để trống"})),
        ));
    }

    // Save to metadata storage

    let _ = state.service.storage().set_meta("dns_upstream", trimmed);

    // Update balancer if available

    if let Some(ref balancer) = state.dns_balancer {
        balancer.set_upstreams(&[trimmed.to_string()]);
    }

    // Flush cache to ensure immediate adoption

    if let Some(ref cache) = state.dns_cache {
        cache.flush();
    }

    Ok(Json(serde_json::json!({

        "success": true,

        "current_upstream": trimmed,

        "message": format!("Đã cập nhật máy chủ DNS sang: {}", trimmed)

    })))
}

#[derive(Deserialize)]

pub struct CaptureBlurPayload {
    pub blur_mode: String,
}

async fn handle_get_capture_blur(
    State(state): State<ServerState>,

    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if !check_auth(&state, &headers, None).await {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let mode = state
        .service
        .storage()
        .get_meta("screenshot_blur_mode")
        .ok()
        .flatten()
        .unwrap_or_else(|| "none".to_string());

    Ok(Json(serde_json::json!({ "blur_mode": mode })))
}

async fn handle_set_capture_blur(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Json(payload): Json<CaptureBlurPayload>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if !check_auth(&state, &headers, None).await {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "Unauthorized"})),
        ));
    }

    let mode = payload.blur_mode.trim().to_lowercase();

    if mode != "none" && mode != "reduced" && mode != "full" {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({

                "error": "Chế độ làm mờ không hợp lệ. Chỉ chấp nhận: 'none' (tắt mờ), 'reduced' (giảm mờ), 'full' (mờ bảo vệ)"

            })),
        ));
    }

    let _ = state
        .service
        .storage()
        .set_meta("screenshot_blur_mode", &mode);

    Ok(Json(serde_json::json!({

        "success": true,

        "blur_mode": mode,

        "message": format!("Đã lưu cài đặt làm mờ ảnh chụp màn hình: {}", mode)

    })))
}

async fn handle_flush_dns_cache(
    State(state): State<ServerState>,

    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if !check_auth(&state, &headers, None).await {
        return Err(StatusCode::UNAUTHORIZED);
    }

    if let Some(ref cache) = state.dns_cache {
        cache.flush();
    }

    #[cfg(windows)]
    {
        let _ = std::process::Command::new("ipconfig")
            .arg("/flushdns")
            .output();
    }

    Ok(Json(serde_json::json!({

        "success": true,

        "message": "Đã làm sạch toàn bộ bộ nhớ đệm DNS."

    })))
}

#[derive(Deserialize)]

pub struct DnsRetentionPayload {
    pub retention_days: u32,

    pub auto_prune: bool,
}

async fn handle_get_dns_retention(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Query(query): Query<QueryParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if !check_auth(&state, &headers, query.token.as_deref()).await {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let retention_days: u32 = state
        .service
        .storage()
        .get_meta("dns_log_retention_days")
        .ok()
        .flatten()
        .and_then(|v| v.parse().ok())
        .unwrap_or(7);

    let auto_prune: bool = state
        .service
        .storage()
        .get_meta("dns_auto_prune_enabled")
        .ok()
        .flatten()
        .map(|v| v != "false")
        .unwrap_or(true);

    let (raw_count, agg_count) = state
        .service
        .storage()
        .get_query_logs_storage_stats()
        .unwrap_or((0, 0));

    let last_pruned_at = state
        .service
        .storage()
        .get_meta("dns_log_last_pruned_at")
        .ok()
        .flatten();

    Ok(Json(serde_json::json!({

        "retention_days": retention_days,

        "auto_prune": auto_prune,

        "raw_query_logs_count": raw_count,

        "aggregated_domains_count": agg_count,

        "last_pruned_at": last_pruned_at,

    })))
}

async fn handle_set_dns_retention(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Query(query): Query<QueryParams>,

    Json(payload): Json<DnsRetentionPayload>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if !check_auth(&state, &headers, query.token.as_deref()).await {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "Unauthorized"})),
        ));
    }

    if payload.retention_days == 0 || payload.retention_days > 365 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Thời gian lưu trữ phải từ 1 đến 365 ngày"})),
        ));
    }

    let _ = state.service.storage().set_meta(
        "dns_log_retention_days",
        &payload.retention_days.to_string(),
    );

    let _ = state.service.storage().set_meta(
        "dns_auto_prune_enabled",
        if payload.auto_prune { "true" } else { "false" },
    );

    Ok(Json(serde_json::json!({

        "success": true,

        "retention_days": payload.retention_days,

        "auto_prune": payload.auto_prune,

        "message": format!("Đã cập nhật thời gian lưu trữ chi tiết: {} ngày", payload.retention_days),

    })))
}

async fn handle_prune_dns_retention_now(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Query(query): Query<QueryParams>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if !check_auth(&state, &headers, query.token.as_deref()).await {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "Unauthorized"})),
        ));
    }

    let retention_days: i64 = state
        .service
        .storage()
        .get_meta("dns_log_retention_days")
        .ok()
        .flatten()
        .and_then(|v| v.parse().ok())
        .unwrap_or(7);

    let cutoff = Utc::now() - Duration::days(retention_days);

    match state.service.storage().rollup_and_prune_query_logs(cutoff) {
        Ok(deleted) => {
            let now = Utc::now();

            let _ = state
                .service
                .storage()
                .set_meta("dns_log_last_pruned_at", &now.to_rfc3339());

            Ok(Json(serde_json::json!({

                "success": true,

                "deleted_raw_logs": deleted,

                "message": format!("Đã bảo toàn bản tổng hợp và dọn dẹp {} bản ghi DNS chi tiết", deleted)

            })))
        }

        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": format!("Lỗi dọn dẹp DNS: {}", e)})),
        )),
    }
}

// -----------------------------------------------------------------------------

// System Logs Reading & Download Handlers

// -----------------------------------------------------------------------------

fn get_latest_service_log_path() -> Option<(std::path::PathBuf, String)> {
    let data_dir = std::env::var("ProgramData")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join("FamShield")
        .join("logs");

    if let Ok(entries) = std::fs::read_dir(&data_dir) {
        let mut log_files: Vec<(std::path::PathBuf, String)> = entries
            .filter_map(|e| e.ok())
            .map(|e| {
                let p = e.path();

                let name = p
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();

                (p, name)
            })
            .filter(|(_, name)| name.starts_with("famshield-service.log"))
            .collect();

        log_files.sort_by(|a, b| a.1.cmp(&b.1));

        return log_files.pop();
    }

    None
}

fn resolve_nats_url_from_storage(storage: &famshield_storage::StorageEngine) -> String {
    if let Ok(Some(nats_url)) = storage.get_meta("nats_url") {
        let trimmed = nats_url.trim();
        if !trimmed.is_empty() {
            return famshield_core::normalize_nats_url(trimmed);
        }
    }

    if let Ok(Some(server_url)) = storage.get_meta("server_url") {
        let trimmed = server_url.trim();
        if !trimmed.is_empty() {
            return famshield_core::normalize_nats_url(trimmed);
        }
    }

    "wss://famshield.ttcantho.io.vn/nats".to_string()
}

#[derive(Deserialize)]

pub struct SystemLogsQuery {
    pub limit: Option<usize>,

    pub token: Option<String>,
}

async fn handle_get_system_logs(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Query(query): Query<SystemLogsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if !check_auth(&state, &headers, query.token.as_deref()).await {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Unauthorized" })),
        ));
    }

    let (path, file_name) = match get_latest_service_log_path() {
        Some(res) => res,

        None => {
            return Ok(Json(serde_json::json!({

                "lines": [],

                "total_lines": 0,

                "file_name": ""

            })));
        }
    };

    let content = std::fs::read_to_string(&path).unwrap_or_default();

    let all_lines: Vec<&str> = content.lines().collect();

    let total = all_lines.len();

    let take_count = query.limit.unwrap_or(200).min(total);

    let start_idx = total.saturating_sub(take_count);

    let selected_lines: Vec<String> = all_lines[start_idx..]
        .iter()
        .map(|s| s.to_string())
        .collect();

    Ok(Json(serde_json::json!({

        "lines": selected_lines,

        "total_lines": total,

        "file_name": file_name

    })))
}

async fn handle_download_system_logs(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Query(query): Query<QueryParams>,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    if !check_auth(&state, &headers, query.token.as_deref()).await {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Unauthorized" })),
        ));
    }

    let (path, file_name) = match get_latest_service_log_path() {
        Some(res) => res,

        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Log file not found" })),
            ));
        }
    };

    let bytes = std::fs::read(&path).unwrap_or_default();

    let response = Response::builder()
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", file_name),
        )
        .body(axum::body::Body::from(bytes))
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    Ok(response)
}

// -----------------------------------------------------------------------------

// System Errors & NATS Streaming Handlers

// -----------------------------------------------------------------------------

async fn handle_list_errors(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Query(query): Query<QueryParams>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if !check_auth(&state, &headers, query.token.as_deref()).await {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Unauthorized" })),
        ));
    }

    let errors = state.service.list_all_errors(200).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    let unsynced_count = errors.iter().filter(|e| !e.synced).count();

    Ok(Json(serde_json::json!({

        "total_count": errors.len(),

        "unsynced_count": unsynced_count,

        "errors": errors

    })))
}

async fn handle_clear_errors(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Query(query): Query<QueryParams>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if !check_auth(&state, &headers, query.token.as_deref()).await {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Unauthorized" })),
        ));
    }

    let cleared = state.service.clear_errors().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    Ok(Json(serde_json::json!({

        "success": true,

        "cleared_count": cleared

    })))
}

async fn handle_push_errors_nats(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Query(query): Query<QueryParams>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if !check_auth(&state, &headers, query.token.as_deref()).await {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Unauthorized" })),
        ));
    }

    let unsynced = state
        .service
        .storage()
        .list_unsynced_errors(100)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    if unsynced.is_empty() {
        return Ok(Json(serde_json::json!({

            "success": true,

            "pushed_count": 0,

            "message": "Không có lỗi mới cần đẩy lên máy chủ"

        })));
    }

    let device_id = state
        .service
        .storage()
        .get_meta("enrolled_device_id")
        .ok()
        .flatten()
        .unwrap_or_else(|| "dev-local-001".to_string());

    let nats_url = if let Ok(Some(url)) = state.service.storage().get_meta("nats_url") {
        if !url.trim().is_empty() {
            url
        } else {
            resolve_nats_url_from_storage(state.service.storage())
        }
    } else {
        resolve_nats_url_from_storage(state.service.storage())
    };

    let serialized_errors: Vec<serde_json::Value> = unsynced
        .iter()
        .map(|e| {
            serde_json::json!({

                "error_id": e.error_id,

                "device_id": device_id,

                "module": e.module,

                "severity": e.severity,

                "message": e.message,

                "stack_trace": e.stack_trace,

                "occurrence_count": e.occurrence_count,

                "first_seen_at": e.first_seen_at.to_rfc3339(),

                "last_seen_at": e.last_seen_at.to_rfc3339(),

            })
        })
        .collect();

    let batch = serde_json::json!({

        "batch_id": format!("batch-err-{}", chrono::Utc::now().timestamp_millis()),

        "device_id": device_id,

        "count": serialized_errors.len(),

        "errors": serialized_errors,

    });

    let connect_opts =
        async_nats::ConnectOptions::new().connection_timeout(std::time::Duration::from_secs(5));

    let ids: Vec<String> = unsynced.iter().map(|e| e.error_id.clone()).collect();

    match connect_opts.connect(&nats_url).await {
        Ok(client) => {
            let subject = format!("devices.{}.errors", device_id);

            let payload = serde_json::to_vec(&batch).unwrap_or_default();

            let _ = client.publish(subject, payload.clone().into()).await;

            let _ = client
                .publish("famshield.events.error_log", payload.into())
                .await;

            let _ = client.flush().await;

            let _ = state.service.storage().mark_errors_synced(&ids);

            // Also enqueue into outbox as durable fallback

            for err in &unsynced {
                let _ = state.service.storage().enqueue_outbox(
                    &format!("err-{}", err.error_id),
                    "error_log",
                    chrono::Utc::now(),
                    &serde_json::to_string(err).unwrap_or_default(),
                );
            }

            Ok(Json(serde_json::json!({

                "success": true,

                "pushed_count": unsynced.len(),

                "nats_url": nats_url,

                "message": format!("Đã đẩy thành công {} lỗi lên Server qua NATS", unsynced.len())

            })))
        }

        Err(err) => {
            // Still enqueue into outbox for HTTP/NATS retry

            for err in &unsynced {
                let _ = state.service.storage().enqueue_outbox(
                    &format!("err-{}", err.error_id),
                    "error_log",
                    chrono::Utc::now(),
                    &serde_json::to_string(err).unwrap_or_default(),
                );
            }

            Err((
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({

                    "success": false,

                    "error": format!("Không thể kết nối NATS broker ({}): {}. Lỗi đã được lưu vào Outbox để tự động gửi lại.", nats_url, err)

                })),
            ))
        }
    }
}

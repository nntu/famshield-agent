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
    probe_server_connection, ActivitySummaryResponse, AddKeywordRequest, BackupPayload,
    CreateRuleRequest, KeywordConfigResponse, KeywordRuleDto, LocalAdminService, ServerConfig,
    ToggleKeywordRequest, UpdateServerConfigRequest,
};
use crate::ui::LOCAL_DASHBOARD_HTML;
use famshield_dns::blocklist::BlocklistStore;
use famshield_storage::{
    KeywordViolationRecord, LocalRuleRecord, QueryLogRecord, UsageSegmentRecord,
};

#[derive(Clone)]
pub struct ServerState {
    pub service: Arc<LocalAdminService>,
    pub dns_blocklist: Option<Arc<BlocklistStore>>,
    pub sessions: Arc<Mutex<HashMap<String, DateTime<Utc>>>>,
    pub active_app: Arc<Mutex<String>>,
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
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct VerifyResponse {
    pub authenticated: bool,
}

#[derive(Serialize)]
pub struct StatusResponse {
    pub protection_active: bool,
    pub monitor_mode: bool,
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
}

#[derive(Deserialize)]
pub struct ProtectionModeRequest {
    pub monitor_mode: bool,
}

#[derive(Serialize)]
pub struct ProtectionModeResponse {
    pub success: bool,
    pub monitor_mode: bool,
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
                sessions: Arc::new(Mutex::new(HashMap::new())),
                active_app: Arc::new(Mutex::new("Roblox.exe".to_string())),
            },
        }
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
            .route("/api/auth/logout", post(handle_logout))
            .route("/api/auth/verify", get(handle_verify))
            .route("/api/status", get(handle_status))
            .route("/api/activity/summary", get(handle_get_activity_summary))
            .route(
                "/api/rules",
                get(handle_list_rules).post(handle_create_rule),
            )
            .route("/api/rules/bulk", post(handle_bulk_create_rules))
            .route("/api/rules/:rule_id", delete(handle_delete_rule))
            .route("/api/logs/dns", get(handle_list_logs))
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
                "/api/settings/protection-mode",
                get(handle_get_protection_mode).post(handle_set_protection_mode),
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
}

fn find_dist_dir() -> Option<std::path::PathBuf> {
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
                    error: Some(format!("Lỗi xác thực: {}", e)),
                }),
            )
                .into_response()
        }
    }
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

    let session_helper_running = famshield_windows::SessionManager::is_session_helper_running();
    let session_helper_status = if session_helper_running {
        "RUNNING".to_string()
    } else {
        "STOPPED".to_string()
    };

    Ok(Json(StatusResponse {
        protection_active: true,
        monitor_mode,
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

async fn handle_list_logs(
    State(state): State<ServerState>,
    headers: HeaderMap,
    Query(query): Query<QueryParams>,
) -> Result<Json<Vec<QueryLogRecord>>, StatusCode> {
    if !check_auth(&state, &headers, query.token.as_deref()).await {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let limit = query.limit.unwrap_or(100).min(500);
    match state.service.list_query_logs(limit) {
        Ok(logs) => Ok(Json(logs)),
        Err(e) => {
            warn!("Failed to query DNS logs: {}", e);
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
            Json(serde_json::json!({ "error": "Mã ghép nối phải gồm 6 chữ số hợp lệ" })),
        ));
    }

    let dev_id = format!("cloud-dev-{}", &code[..4]);
    let _ = state
        .service
        .storage()
        .set_meta("enrolled_device_id", &dev_id);
    let _ = state
        .service
        .storage()
        .set_meta("enrolled_tenant_id", "tenant-famshield-pro");

    info!("Device linked to Cloud with code: {}", code);
    Ok(Json(serde_json::json!({
        "success": true,
        "operating_mode": "OnlineManaged",
        "device_id": dev_id
    })))
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
            if !trimmed.is_empty() {
                let _ = state.service.storage().set_meta("enrollment_code", trimmed);
                let dev_id = format!("cloud-dev-{}", trimmed);
                let _ = state
                    .service
                    .storage()
                    .set_meta("enrolled_device_id", &dev_id);
                let _ = state
                    .service
                    .storage()
                    .set_meta("enrolled_tenant_id", "tenant-famshield-pro");
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
    Ok(Json(ProtectionModeResponse {
        success: true,
        monitor_mode,
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

    if let Err(e) = state.service.set_monitor_mode(req.monitor_mode) {
        warn!("Failed to set monitor mode: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    sync_dns_blocklist(&state);

    info!(
        "Protection mode updated: monitor_mode = {}",
        req.monitor_mode
    );

    Ok(Json(ProtectionModeResponse {
        success: true,
        monitor_mode: req.monitor_mode,
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
    Json(payload): Json<RecordViolationRequest>,
) -> Result<StatusCode, StatusCode> {
    let id = format!("kw_v_{}", Utc::now().timestamp_millis());
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

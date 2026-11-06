use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use tracing::warn;

use super::dns::{get_latest_service_log_path, resolve_nats_url_from_storage, SystemLogsQuery};
use crate::server::check_auth;
use crate::server::types::*;
use crate::server::ServerState;

pub(crate) async fn handle_status(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Query(query): Query<QueryParams>,
) -> Result<Json<StatusResponse>, StatusCode> {
    if !check_auth(&state, &headers, query.token.as_deref()).await {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let (total_queries_today, blocked_queries_today) = state
        .service
        .storage()
        .get_dns_query_stats_today()
        .unwrap_or_else(|_| {
            let logs = state.service.list_query_logs(200).unwrap_or_default();
            let blocked = logs.iter().filter(|l| l.action == "BLOCK").count();
            (logs.len(), blocked)
        });

    let rules_count = state.service.list_rules().map(|r| r.len()).unwrap_or(0);

    let evidence_count = state
        .service
        .storage()
        .count_visual_evidence()
        .unwrap_or_else(|_| {
            state
                .service
                .list_visual_evidence(50)
                .map(|e| e.len())
                .unwrap_or(0)
        });

    let used_today_ms = state
        .service
        .storage()
        .get_total_screen_time_today_ms()
        .unwrap_or(0);
    let used_today_minutes = (used_today_ms / 60_000).max(0) as u32;

    let daily_quota_minutes = state
        .service
        .storage()
        .get_meta("daily_quota_minutes")
        .ok()
        .flatten()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(120);

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

    let nats_status = state
        .service
        .storage()
        .get_meta("nats_status")
        .ok()
        .flatten()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "disconnected".to_string());

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

        daily_quota_minutes,

        used_today_minutes,

        blocked_queries_today,

        total_queries_today,

        rules_count,

        evidence_count,

        app_version: env!("CARGO_PKG_VERSION").to_string(),

        nats_status,

        system_info: Some(famshield_windows::get_system_info()),
    }))
}

pub(crate) async fn handle_get_system_info(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Query(query): Query<QueryParams>,
) -> Result<Json<famshield_windows::SystemInfo>, StatusCode> {
    if !check_auth(&state, &headers, query.token.as_deref()).await {
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(Json(famshield_windows::get_system_info()))
}

pub(crate) async fn handle_list_evidence(
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

pub(crate) async fn handle_get_thumbnail(
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

pub(crate) async fn handle_start_session_helper(
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

pub(crate) async fn handle_get_update_urls(State(state): State<ServerState>) -> impl IntoResponse {
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

pub(crate) async fn handle_update_urls(
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

pub(crate) async fn handle_check_update(State(state): State<ServerState>) -> impl IntoResponse {
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
pub struct UninstallPayload {
    pub master_pin: String,

    pub otp_code: Option<String>,
}

pub(crate) async fn handle_system_uninstall(
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
                            let computer_name = famshield_windows::get_computer_name();

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

#[derive(serde::Deserialize)]
pub struct RestartServicePayload {
    pub master_pin: Option<String>,
    pub otp_code: Option<String>,
}

pub(crate) async fn handle_restart_service(
    State(state): State<ServerState>,
    headers: HeaderMap,
    Json(payload): Json<RestartServicePayload>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let is_web_authenticated = check_auth(&state, &headers, None).await;

    if !is_web_authenticated {
        let pin = payload
            .master_pin
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty());

        let Some(entered_pin) = pin else {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "success": false,
                    "error": "Yêu cầu nhập Master PIN hoặc Mã xác thực của cha mẹ"
                })),
            ));
        };

        match state.service.authenticate_storage(entered_pin) {
            Ok(true) => {
                // Check 2FA OTP if enabled
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
                                    "message": "Xác thực 2 lớp (2FA) đang bật. Vui lòng nhập thêm mã OTP 6 số hoặc mã dự phòng để xác nhận khởi động lại dịch vụ."
                                })),
                            ));
                        };

                        let mut valid = false;
                        if otp.len() == 6 && otp.chars().all(|c| c.is_ascii_digit()) {
                            if let Ok(unprotected) =
                                crate::secret_protection::unprotect_secret(&cfg.encrypted_secret)
                            {
                                let computer_name = famshield_windows::get_computer_name();
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
                            let _ = state.service.storage().record_error_log(
                                "auth_restart",
                                "HIGH",
                                "Cảnh báo: Có hành vi cố gắng nhập sai mã 2FA OTP để khởi động lại dịch vụ từ Menu Tray",
                                None,
                            );

                            let now = chrono::Utc::now();
                            let alert_payload = serde_json::json!({
                                "alert_type": "unauthorized_service_restart_attempt",
                                "severity": "HIGH",
                                "reason": "Có hành vi cố gắng nhập sai mã 2FA OTP để khởi động lại dịch vụ",
                                "source": if is_web_authenticated { "web_admin" } else { "tray_menu" },
                                "timestamp": now.to_rfc3339(),
                            });
                            let _ = state.service.storage().enqueue_outbox(
                                &format!("alert-otp-restart-{}", now.timestamp_micros()),
                                "security_alert",
                                now,
                                &alert_payload.to_string(),
                            );

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
            }
            Ok(false) => {
                let _ = state.service.storage().record_error_log(
                    "auth_restart",
                    "HIGH",
                    "Cảnh báo: Có hành vi cố gắng nhập sai Master PIN để khởi động lại dịch vụ từ Menu Tray",
                    None,
                );

                let now = chrono::Utc::now();
                let alert_payload = serde_json::json!({
                    "alert_type": "unauthorized_service_restart_attempt",
                    "severity": "HIGH",
                    "reason": "Có hành vi cố gắng nhập sai Master PIN để khởi động lại dịch vụ",
                    "source": if is_web_authenticated { "web_admin" } else { "tray_menu" },
                    "timestamp": now.to_rfc3339(),
                });
                let _ = state.service.storage().enqueue_outbox(
                    &format!("alert-pin-restart-{}", now.timestamp_micros()),
                    "security_alert",
                    now,
                    &alert_payload.to_string(),
                );

                return Err((
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({
                        "success": false,
                        "wrong_pin": true,
                        "error": "Master PIN không chính xác"
                    })),
                ));
            }
            Err(e) => {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "success": false,
                        "error": format!("Lỗi xác thực: {}", e)
                    })),
                ));
            }
        }
    }

    let now = chrono::Utc::now();
    let _ = state.service.storage().record_error_log(
        "service_control",
        "INFO",
        "Khởi động lại FamShield Service thành công theo yêu cầu đã xác thực của phụ huynh",
        None,
    );

    let alert_payload = serde_json::json!({
        "alert_type": "service_restarted_by_parent",
        "severity": "INFO",
        "reason": "Phụ huynh đã xác thực danh tính và khởi động lại FamShield Service",
        "source": if is_web_authenticated { "web_admin" } else { "tray_menu" },
        "timestamp": now.to_rfc3339(),
    });
    let _ = state.service.storage().enqueue_outbox(
        &format!("alert-restart-ok-{}", now.timestamp_micros()),
        "security_alert",
        now,
        &alert_payload.to_string(),
    );

    tracing::info!("Parent authenticated for Service Restart. Spawning restart command...");

    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        let _ = famshield_windows::restart_service("FamShieldService");
    });

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Đã xác thực thành công. FamShield Service đang khởi động lại..."
    })))
}

pub(crate) async fn handle_get_system_logs(
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

pub(crate) async fn handle_download_system_logs(
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

pub(crate) async fn handle_list_errors(
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

pub(crate) async fn handle_clear_errors(
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

pub(crate) async fn handle_push_errors_nats(
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

    // Persist the HTTP telemetry fallback before attempting the best-effort NATS push.
    for error in &unsynced {
        let payload = serde_json::to_string(error).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;
        state
            .service
            .storage()
            .enqueue_outbox(
                &format!("err-{}", error.error_id),
                "error_log",
                chrono::Utc::now(),
                &payload,
            )
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": e.to_string() })),
                )
            })?;
    }

    match connect_opts.connect(&nats_url).await {
        Ok(client) => {
            let subject = format!("devices.{}.errors", device_id);

            let payload = serde_json::to_vec(&batch).map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": e.to_string() })),
                )
            })?;

            client
                .publish(subject, payload.clone().into())
                .await
                .map_err(|e| {
                    (
                        StatusCode::BAD_GATEWAY,
                        Json(serde_json::json!({ "error": format!("NATS publish failed: {}", e) })),
                    )
                })?;

            client
                .publish("famshield.events.error_log", payload.into())
                .await
                .map_err(|e| {
                    (
                        StatusCode::BAD_GATEWAY,
                        Json(serde_json::json!({ "error": format!("NATS publish failed: {}", e) })),
                    )
                })?;

            client.flush().await.map_err(|e| {
                (
                    StatusCode::BAD_GATEWAY,
                    Json(serde_json::json!({ "error": format!("NATS flush failed: {}", e) })),
                )
            })?;

            state
                .service
                .storage()
                .mark_errors_synced(&ids)
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({ "error": e.to_string() })),
                    )
                })?;

            Ok(Json(serde_json::json!({

                "success": true,

                "pushed_count": unsynced.len(),

                "nats_url": nats_url,

                "message": format!("Đã đẩy thành công {} lỗi lên Server qua NATS", unsynced.len())

            })))
        }

        Err(err) => Err((
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({

                "success": false,

                "error": format!("Không thể kết nối NATS broker ({}): {}. Lỗi đã được lưu vào Outbox để tự động gửi lại.", nats_url, err)

            })),
        )),
    }
}

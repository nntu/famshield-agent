use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use tracing::{info, warn};

use super::rules::PauseFilterPayload;
use crate::server::check_auth;
use crate::server::types::*;
use crate::server::ServerState;
use famshield_storage::{DomainQuerySummary, QueryLogRecord};

pub(crate) async fn handle_filter_pause(
    State(state): State<ServerState>,
    headers: HeaderMap,
    Query(query): Query<QueryParams>,
    Json(body): Json<PauseFilterPayload>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if !check_auth(&state, &headers, query.token.as_deref()).await {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Unauthorized" })),
        ));
    }

    let secs = body.seconds.unwrap_or(900);
    if let Some(bl) = &state.dns_blocklist {
        bl.pause(secs);
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let paused_until = now + secs;
    let _ = state
        .service
        .storage()
        .set_meta("filter_paused_until", &paused_until.to_string());

    info!(
        "DNS & Web filtering paused for {} seconds (until {})",
        secs, paused_until
    );

    Ok(Json(serde_json::json!({
        "success": true,
        "paused_until": paused_until,
        "remaining_seconds": secs,
        "message": format!("Đã tạm dừng lọc web trong {} giây", secs)
    })))
}

pub(crate) async fn handle_filter_unpause(
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

    if let Some(bl) = &state.dns_blocklist {
        bl.unpause();
    }
    let _ = state.service.storage().set_meta("filter_paused_until", "0");

    info!("DNS & Web filtering unpaused (protection active)");

    Ok(Json(serde_json::json!({
        "success": true,
        "paused_until": 0,
        "remaining_seconds": 0,
        "message": "Đã tiếp tục kích hoạt lại chế độ bảo vệ web & DNS"
    })))
}

#[derive(Deserialize)]
pub(crate) struct InspectDomainPayload {
    domain: String,
}

pub(crate) async fn handle_filter_inspect(
    State(state): State<ServerState>,
    headers: HeaderMap,
    Query(query): Query<QueryParams>,
    Json(body): Json<InspectDomainPayload>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if !check_auth(&state, &headers, query.token.as_deref()).await {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Unauthorized" })),
        ));
    }

    let raw = body
        .domain
        .trim()
        .trim_end_matches('.')
        .to_ascii_lowercase();
    if raw.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Tên miền không được để trống" })),
        ));
    }

    let is_blocked = if let Some(bl) = &state.dns_blocklist {
        bl.is_blocked(&raw)
    } else {
        false
    };

    let reason = if let Some(bl) = &state.dns_blocklist {
        bl.block_reason(&raw)
            .map(|r| r.to_string())
            .unwrap_or_else(|| {
                if is_blocked {
                    "Bị chặn theo chính sách lọc".to_string()
                } else {
                    "Được phép truy cập (hợp lệ hoặc nằm trong Whitelist)".to_string()
                }
            })
    } else if is_blocked {
        "Bị chặn".to_string()
    } else {
        "Được phép".to_string()
    };

    let action = if is_blocked { "block" } else { "allow" };
    let effective_cat = if is_blocked {
        "Danh mục cấm / Blacklist"
    } else {
        "Hợp lệ / Whitelist"
    };

    Ok(Json(serde_json::json!({
        "domain": raw,
        "action": action,
        "effective_category": effective_cat,
        "reason": reason
    })))
}

pub(crate) async fn handle_list_logs(
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

pub(crate) async fn handle_get_dns_summary(
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

pub(crate) async fn handle_get_dns_hourly(
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

pub(crate) async fn handle_get_dns_config(
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

pub(crate) async fn handle_set_dns_config(
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

pub(crate) async fn handle_flush_dns_cache(
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

pub(crate) async fn handle_get_dns_retention(
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

pub(crate) async fn handle_set_dns_retention(
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

pub(crate) async fn handle_prune_dns_retention_now(
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

pub(crate) fn get_latest_service_log_path() -> Option<(std::path::PathBuf, String)> {
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

pub(crate) fn resolve_nats_url_from_storage(storage: &famshield_storage::StorageEngine) -> String {
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

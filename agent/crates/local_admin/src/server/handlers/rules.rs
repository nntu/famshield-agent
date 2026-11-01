use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use tracing::{info, warn};

use crate::server::types::*;
use crate::server::ServerState;
use crate::server::{check_auth, extract_token, sync_dns_blocklist};
use crate::service::{CreateRuleRequest, UpdateServerConfigRequest};
use famshield_storage::LocalRuleRecord;

pub(crate) async fn handle_list_rules(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Query(query): Query<QueryParams>,
) -> Result<Json<Vec<LocalRuleRecord>>, StatusCode> {
    if !check_auth(&state, &headers, query.token.as_deref()).await {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let mut rules = match state.service.list_rules() {
        Ok(rules) => rules,
        Err(e) => {
            warn!("Failed to query local rules: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Enrich with cloud custom blocked domains from cached_cloud_policy
    if let Ok(Some(cached)) = state.service.storage().get_meta("cached_cloud_policy") {
        if let Ok(cloud_policy) = serde_json::from_str::<serde_json::Value>(&cached) {
            let ver = cloud_policy
                .get("version")
                .and_then(|v| v.as_u64())
                .unwrap_or(1);
            if let Some(domains) = cloud_policy
                .get("custom_blocked_domains")
                .and_then(|v| v.as_array())
            {
                let local_set: std::collections::HashSet<String> = rules
                    .iter()
                    .filter(|r| r.rule_type == "domain")
                    .map(|r| r.rule_id.to_lowercase())
                    .collect();

                let now = Utc::now();
                for d_val in domains {
                    if let Some(domain) = d_val.as_str() {
                        let trimmed = domain.trim().to_lowercase();
                        if !trimmed.is_empty() && !local_set.contains(&trimmed) {
                            rules.push(LocalRuleRecord {
                                rule_id: trimmed,
                                rule_type: "domain".to_string(),
                                payload: serde_json::json!({
                                    "reason": format!("Chính sách từ Máy chủ Đám mây (Cloud Policy v{})", ver)
                                }).to_string(),
                                enabled: true,
                                created_at: now,
                                updated_at: now,
                                source: "cloud".to_string(),
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(Json(rules))
}

pub(crate) async fn handle_create_rule(
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

pub(crate) async fn handle_bulk_create_rules(
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

pub(crate) async fn handle_delete_rule(
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

pub(crate) async fn handle_pull_rules_from_cloud(
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
                "error": "Thiết bị chưa được cấu hình địa chỉ FamShield Server."
            })),
        ));
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());

    let policy_url = format!(
        "{}/v1/filter/policy?tenant_id={}",
        server_url.trim_end_matches('/'),
        tenant_id
    );

    let res = client.get(&policy_url).send().await.map_err(|e| {
        (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({
                "error": format!("Không thể kết nối đến Cloud Server để kéo chính sách: {}", e)
            })),
        )
    })?;

    if !res.status().is_success() {
        return Err((
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({
                "error": format!("Máy chủ phản hồi mã lỗi: {}", res.status())
            })),
        ));
    }

    let policy_json: serde_json::Value = res.json().await.map_err(|e| {
        (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({
                "error": format!("Dữ liệu chính sách từ máy chủ không hợp lệ: {}", e)
            })),
        )
    })?;

    // Cache the policy in storage
    let policy_str = policy_json.to_string();
    let _ = state
        .service
        .storage()
        .set_meta("cached_cloud_policy", &policy_str);

    let ver = policy_json
        .get("version")
        .and_then(|v| v.as_u64())
        .unwrap_or(1);
    let _ = state
        .service
        .storage()
        .set_meta("policy_version", &ver.to_string());
    let _ = state
        .service
        .storage()
        .set_meta("last_policy_sync", &chrono::Utc::now().to_rfc3339());

    let domains_count = policy_json
        .get("custom_blocked_domains")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);

    // Refresh active DNS sinkhole blocklist
    sync_dns_blocklist(&state);

    info!(
        "Successfully pulled latest cloud policy v{} ({} custom domains) for tenant '{}'",
        ver, domains_count, tenant_id
    );

    Ok(Json(serde_json::json!({
        "success": true,
        "policy_version": ver,
        "cloud_domains_count": domains_count,
        "policy": policy_json,
        "message": format!("Đã kéo và áp dụng thành công Chính sách Cloud v{} ({} tên miền)", ver, domains_count)
    })))
}

pub(crate) async fn handle_get_cloud_policy(
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

    let paused_until = if let Some(bl) = &state.dns_blocklist {
        if bl.is_paused() {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            now + bl.paused_remaining_secs()
        } else {
            0
        }
    } else {
        0
    };

    if let Ok(Some(cached)) = state.service.storage().get_meta("cached_cloud_policy") {
        if let Ok(mut val) = serde_json::from_str::<serde_json::Value>(&cached) {
            if let Some(obj) = val.as_object_mut() {
                obj.insert("paused_until".to_string(), serde_json::json!(paused_until));
            }
            return Ok(Json(val));
        }
    }

    Ok(Json(serde_json::json!({
        "version": 1,
        "block_adult": false,
        "block_gambling": false,
        "block_violence": false,
        "block_social": false,
        "block_gaming": false,
        "safesearch_enabled": false,
        "youtube_restricted": false,
        "paused_until": paused_until,
        "custom_blocked_domains": [],
        "custom_allowed_domains": []
    })))
}

#[derive(Deserialize)]
struct PauseFilterPayload {
    seconds: Option<u64>,
}


pub(crate) async fn handle_sync_rules_to_cloud(
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


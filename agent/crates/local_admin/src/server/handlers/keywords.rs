use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::server::types::*;
use crate::server::ServerState;
use crate::server::{check_auth, extract_token};
use crate::service::{AddKeywordRequest, KeywordConfigResponse, KeywordRuleDto, ToggleKeywordRequest};
use famshield_storage::KeywordViolationRecord;

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

pub(crate) async fn handle_get_keywords(
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

pub(crate) async fn handle_add_keyword(
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

pub(crate) async fn handle_delete_keyword(
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

pub(crate) async fn handle_toggle_keyword(
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

pub(crate) async fn handle_get_keyboard_monitoring(
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

pub(crate) async fn handle_toggle_keyboard_monitoring(
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

pub(crate) async fn handle_record_keyword_violation(
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


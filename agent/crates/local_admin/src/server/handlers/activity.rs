use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::Deserialize;
use tracing::warn;

use crate::server::check_auth;
use crate::server::types::*;
use crate::server::ServerState;
use crate::service::{
    ActivitySummaryResponse, AppCategoryMetadata, AppUsageSummary, TimeAggregatedReport,
};

pub(crate) async fn handle_get_activity_summary(
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

pub(crate) async fn handle_get_apps_summary(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Query(query): Query<QueryParams>,
) -> Result<Json<Vec<AppUsageSummary>>, StatusCode> {
    if !check_auth(&state, &headers, query.token.as_deref()).await {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let active_app = state.active_app.lock().await.clone();
    let exclude_sys = query.exclude_system.unwrap_or(false);

    match state.service.get_grouped_app_usage_filtered(
        &active_app,
        query.start_time.as_deref(),
        query.end_time.as_deref(),
        exclude_sys,
    ) {
        Ok(apps) => Ok(Json(apps)),

        Err(e) => {
            warn!("Failed to retrieve grouped app usage: {}", e);

            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub(crate) async fn handle_get_app_categories(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Query(query): Query<QueryParams>,
) -> Result<Json<Vec<AppCategoryMetadata>>, StatusCode> {
    if !check_auth(&state, &headers, query.token.as_deref()).await {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let cats = state
        .service
        .get_app_categories(query.start_time.as_deref(), query.end_time.as_deref());
    Ok(Json(cats))
}

pub(crate) async fn handle_get_time_report(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Query(query): Query<QueryParams>,
) -> Result<Json<TimeAggregatedReport>, StatusCode> {
    if !check_auth(&state, &headers, query.token.as_deref()).await {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let period = query.period.as_deref().unwrap_or("today");
    let exclude_sys = query.exclude_system.unwrap_or(false);

    match state.service.get_time_aggregated_report(
        period,
        query.start_time.as_deref(),
        query.end_time.as_deref(),
        exclude_sys,
    ) {
        Ok(report) => Ok(Json(report)),

        Err(e) => {
            warn!("Failed to generate time-aggregated report: {}", e);

            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub(crate) async fn handle_sync_apps_to_server(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Query(query): Query<QueryParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if !check_auth(&state, &headers, query.token.as_deref()).await {
        return Err(StatusCode::UNAUTHORIZED);
    }

    match state.service.sync_apps_to_server() {
        Ok(resp) => Ok(Json(resp)),

        Err(e) => {
            warn!("Failed to sync apps to server: {}", e);

            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Deserialize)]
pub struct ResetReportsPayload {
    pub master_pin: String,
    pub otp_code: Option<String>,
    pub scope: Option<Vec<String>>,
}

pub(crate) async fn handle_reset_reports(
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

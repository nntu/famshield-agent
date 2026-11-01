use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::{Duration, Utc};
use std::sync::Arc;
use tracing::{info, warn};

use crate::server::types::*;
use crate::server::ServerState;
use crate::server::{check_auth, extract_token, generate_token};

pub(crate) async fn handle_login(
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

pub(crate) async fn handle_login_totp(
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
            let computer_name = famshield_windows::get_computer_name();

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

pub(crate) async fn handle_get_totp_status(
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

pub(crate) async fn handle_setup_totp(
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

    let computer_name = famshield_windows::get_computer_name();

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

pub(crate) async fn handle_confirm_totp(
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

    let computer_name = famshield_windows::get_computer_name();

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

pub(crate) async fn handle_disable_totp(
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

pub(crate) async fn handle_logout(
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

pub(crate) async fn handle_verify(
    State(state): State<ServerState>,

    headers: HeaderMap,

    Query(query): Query<QueryParams>,
) -> impl IntoResponse {
    let auth = check_auth(&state, &headers, query.token.as_deref()).await;

    Json(VerifyResponse {
        authenticated: auth,
    })
}


pub(crate) async fn handle_change_pin(
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


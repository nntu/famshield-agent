use chrono::{Duration, TimeZone, Utc};
use famshield_local_admin::{
    auth::{AuthError, MasterPinAuthenticator, META_PIN_HASH, META_RECOVERY_KEY_HASH},
    service::{CreateRuleRequest, LocalAdminService},
};
use famshield_storage::StorageEngine;

#[test]
fn test_master_pin_hash_and_verify() {
    let pin = "123456";
    let hash = MasterPinAuthenticator::hash_pin(pin).unwrap();

    assert!(MasterPinAuthenticator::verify_pin(pin, &hash).unwrap());
    assert!(!MasterPinAuthenticator::verify_pin("654321", &hash).unwrap());
}

#[test]
fn test_local_admin_rule_creation_and_listing() {
    let storage = StorageEngine::in_memory().unwrap();
    let service = LocalAdminService::new(storage);

    let req = CreateRuleRequest {
        rule_id: "rule-study-mode".into(),
        rule_type: "domain".into(),
        payload_json: r#"{"domains": ["games.com"]}"#.into(),
        enabled: true,
    };

    service.create_rule(req).unwrap();

    let list = service.list_rules().unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].rule_id, "rule-study-mode");

    let deleted = service.delete_rule("rule-study-mode").unwrap();
    assert!(deleted);
    assert_eq!(service.list_rules().unwrap().len(), 0);
}

#[test]
fn test_master_pin_reset_workflows() {
    let storage = StorageEngine::in_memory().unwrap();
    let service = LocalAdminService::new(storage.clone());

    // 1. No insecure default PIN is accepted before guardian setup.
    assert!(matches!(
        service.authenticate_storage("1234"),
        Err(famshield_local_admin::LocalAdminError::Auth(
            AuthError::NotConfigured
        ))
    ));

    // 2. Set new PIN "5678" -> Generates Recovery Key
    let recovery_key = service.set_master_pin("5678").unwrap();
    assert!(recovery_key.starts_with("FAM-"));
    assert_eq!(recovery_key.len(), 18); // "FAM-XXXX-XXXX-XXXX"
    assert!(!storage
        .get_meta(META_PIN_HASH)
        .unwrap()
        .unwrap()
        .starts_with("$argon2"));
    assert!(!storage
        .get_meta(META_RECOVERY_KEY_HASH)
        .unwrap()
        .unwrap()
        .starts_with("$argon2"));

    // Verify new PIN works and old default fails
    assert!(service.authenticate_storage("5678").unwrap());
    assert!(!service.authenticate_storage("1234").unwrap());

    // 3. Reset PIN using Recovery Key -> Reset to "9012"
    service
        .reset_pin_with_recovery_key(&recovery_key, "9012")
        .unwrap();
    assert!(service.authenticate_storage("9012").unwrap());
    assert!(!service.authenticate_storage("5678").unwrap());

    // Attempt reset with invalid recovery key fails
    let err = service.reset_pin_with_recovery_key("FAM-INVALID-KEY", "0000");
    assert!(err.is_err());

    // 4. Elevated Administrator Reset (UAC / Windows Admin CLI) -> Reset to "4444"
    service.reset_pin_elevated("4444").unwrap();
    assert!(service.authenticate_storage("4444").unwrap());
    assert!(!service.authenticate_storage("9012").unwrap());
}

#[test]
fn test_legacy_pin_hash_is_migrated_after_successful_authentication() {
    let storage = StorageEngine::in_memory().unwrap();
    let legacy_hash = MasterPinAuthenticator::hash_pin("8642").unwrap();
    storage.set_meta(META_PIN_HASH, &legacy_hash).unwrap();

    assert!(MasterPinAuthenticator::verify_storage_pin(&storage, "8642").unwrap());
    let migrated = storage.get_meta(META_PIN_HASH).unwrap().unwrap();
    assert_ne!(migrated, legacy_hash);
    assert!(MasterPinAuthenticator::verify_storage_pin(&storage, "8642").unwrap());
}

#[test]
fn test_master_pin_persistent_exponential_lockout() {
    let storage = StorageEngine::in_memory().unwrap();
    MasterPinAuthenticator::set_master_pin(&storage, "2468").unwrap();
    let now = Utc.with_ymd_and_hms(2026, 10, 19, 12, 0, 0).unwrap();

    for _ in 0..4 {
        assert!(!MasterPinAuthenticator::verify_storage_pin_at(&storage, "0000", now).unwrap());
    }

    let locked = MasterPinAuthenticator::verify_storage_pin_at(&storage, "0000", now);
    assert!(matches!(locked, Err(AuthError::Locked(_))));
    assert!(matches!(
        MasterPinAuthenticator::verify_storage_pin_at(
            &storage,
            "2468",
            now + Duration::seconds(29)
        ),
        Err(AuthError::Locked(_))
    ));

    assert!(MasterPinAuthenticator::verify_storage_pin_at(
        &storage,
        "2468",
        now + Duration::seconds(30)
    )
    .unwrap());

    // Successful authentication clears the persistent throttle state.
    assert!(!MasterPinAuthenticator::verify_storage_pin_at(
        &storage,
        "0000",
        now + Duration::seconds(31)
    )
    .unwrap());
}

#[test]
fn test_service_authenticate_enforces_lockout() {
    let storage = StorageEngine::in_memory().unwrap();
    let service = LocalAdminService::new(storage);
    service.set_master_pin("1234").unwrap();

    assert!(service.authenticate("1234").unwrap());
    for _ in 0..4 {
        assert!(!service.authenticate("9999").unwrap());
    }

    let lockout_err = service.authenticate("9999");
    assert!(matches!(
        lockout_err,
        Err(famshield_local_admin::LocalAdminError::Auth(
            AuthError::Locked(_)
        ))
    ));

    // Even correct PIN is blocked during active lockout window
    let blocked_correct = service.authenticate("1234");
    assert!(matches!(
        blocked_correct,
        Err(famshield_local_admin::LocalAdminError::Auth(
            AuthError::Locked(_)
        ))
    ));
}

#[tokio::test]
async fn test_local_admin_web_dashboard_html() {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use famshield_local_admin::LocalAdminServer;
    use std::sync::Arc;
    use tower::ServiceExt;

    let storage = StorageEngine::in_memory().unwrap();
    let service = Arc::new(LocalAdminService::new(storage));
    let server = LocalAdminServer::new(service, None);
    let app = server.router();

    let response = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
    assert!(body_str.contains("FamShield"));
    assert!(body_str.contains("StandaloneFree"));
    assert!(body_str.contains("127.0.0.2:53"));
}

#[tokio::test]
async fn test_local_admin_web_auth_and_status() {
    use axum::body::Body;
    use axum::http::{header, Request, StatusCode};
    use famshield_local_admin::LocalAdminServer;
    use serde_json::Value;
    use std::sync::Arc;
    use tower::ServiceExt;

    let storage = StorageEngine::in_memory().unwrap();
    let service = Arc::new(LocalAdminService::new(storage));
    service.set_master_pin("123456").unwrap();
    let server = LocalAdminServer::new(service, None);

    // 1. Unauthenticated request to /api/status should return 401 Unauthorized
    let app = server.router();
    let res = app
        .oneshot(
            Request::builder()
                .uri("/api/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    // 2. Login with incorrect PIN should return 401
    let app = server.router();
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/login")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"pin":"000000"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    // 3. Login with correct PIN should return 200 OK + token + Set-Cookie
    let app = server.router();
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/login")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"pin":"123456"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert!(res.headers().contains_key(header::SET_COOKIE));

    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["success"], true);
    let token = json["token"].as_str().unwrap().to_string();
    assert!(!token.is_empty());

    // 4. Verify token via /api/auth/verify
    let app = server.router();
    let res = app
        .oneshot(
            Request::builder()
                .uri("/api/auth/verify")
                .header("X-FamShield-Token", &token)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["authenticated"], true);

    // 5. Query /api/status with valid token
    let app = server.router();
    let res = app
        .oneshot(
            Request::builder()
                .uri("/api/status")
                .header("X-FamShield-Token", &token)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["protection_active"], true);
    assert_eq!(json["dns_proxy_addr"], "127.0.0.2:53");
    assert_eq!(json["operating_mode"], "StandaloneFree");
}

#[tokio::test]
async fn test_local_admin_web_rule_crud_and_dns_sync() {
    use axum::body::Body;
    use axum::http::{header, Request, StatusCode};
    use famshield_dns::blocklist::BlocklistStore;
    use famshield_local_admin::LocalAdminServer;
    use serde_json::Value;
    use std::sync::Arc;
    use tower::ServiceExt;

    let storage = StorageEngine::in_memory().unwrap();
    let service = Arc::new(LocalAdminService::new(storage));
    service.set_master_pin("123456").unwrap();
    let dns_blocklist = Arc::new(BlocklistStore::new());
    let server = LocalAdminServer::new(service, Some(dns_blocklist.clone()));

    let token = server.create_session_token().await;

    // 1. Create a domain rule for tiktok.com
    let app = server.router();
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/rules")
                .header("X-FamShield-Token", &token)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    r#"{"rule_id":"tiktok.com","rule_type":"domain","payload_json":"{}","enabled":true}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    // Verify DNS blocklist was atomically updated in real-time
    assert!(dns_blocklist.is_blocked("tiktok.com"));
    assert!(dns_blocklist.is_blocked("v16.tiktok.com"));
    assert!(!dns_blocklist.is_blocked("google.com"));

    // 2. List rules via /api/rules
    let app = server.router();
    let res = app
        .oneshot(
            Request::builder()
                .uri("/api/rules")
                .header("X-FamShield-Token", &token)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let list: Vec<Value> = serde_json::from_slice(&body).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["rule_id"], "tiktok.com");

    // 3. Delete the rule via DELETE /api/rules/:rule_id
    let app = server.router();
    let res = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/api/rules/tiktok.com")
                .header("X-FamShield-Token", &token)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // Verify DNS blocklist was cleared
    assert!(!dns_blocklist.is_blocked("tiktok.com"));
}

#[tokio::test]
async fn test_local_admin_web_logs_evidence_and_cloud() {
    use axum::body::Body;
    use axum::http::{header, Request, StatusCode};
    use famshield_local_admin::LocalAdminServer;
    use famshield_storage::{QueryLogRecord, VisualEvidenceRecord};
    use serde_json::Value;
    use std::sync::Arc;
    use tower::ServiceExt;

    let storage = StorageEngine::in_memory().unwrap();
    let now = Utc::now();

    // Seed query log
    storage
        .insert_query_logs_batch(&[QueryLogRecord {
            domain: "roblox.com".into(),
            qtype: "A".into(),
            action: "ALLOW".into(),
            reason: "default_action_allow".into(),
            rule_id: None,
            occurred_at: now,
        }])
        .unwrap();

    // Seed dummy visual evidence (simulated JPEG)
    let dummy_jpeg = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46];
    storage
        .save_visual_evidence(&VisualEvidenceRecord {
            evidence_id: "ev-test-1".into(),
            event_id: "evt-01".into(),
            occurred_at: now,
            trigger: "Late night gaming block".into(),
            image_payload: dummy_jpeg.clone(),
            size_bytes: dummy_jpeg.len(),
            expires_at: now + Duration::hours(48),
        })
        .unwrap();

    let service = Arc::new(LocalAdminService::new(storage.clone()));
    let server = LocalAdminServer::new(service, None);
    let token = server.create_session_token().await;

    // 1. Fetch DNS logs
    let app = server.router();
    let res = app
        .oneshot(
            Request::builder()
                .uri("/api/logs/dns?limit=10")
                .header("X-FamShield-Token", &token)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let logs: Vec<Value> = serde_json::from_slice(&body).unwrap();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0]["domain"], "roblox.com");

    // 2. Fetch evidence list
    let app = server.router();
    let res = app
        .oneshot(
            Request::builder()
                .uri("/api/evidence")
                .header("X-FamShield-Token", &token)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let ev_list: Vec<Value> = serde_json::from_slice(&body).unwrap();
    assert_eq!(ev_list.len(), 1);
    assert_eq!(ev_list[0]["evidence_id"], "ev-test-1");

    // 3. Fetch evidence thumbnail binary JPEG
    let app = server.router();
    let res = app
        .oneshot(
            Request::builder()
                .uri("/api/evidence/ev-test-1/thumbnail")
                .header("X-FamShield-Token", &token)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(
        res.headers().get(header::CONTENT_TYPE).unwrap(),
        "image/jpeg"
    );
    let img_bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(img_bytes.as_ref(), dummy_jpeg.as_slice());

    // 4. Test Cloud Pairing & Disenroll
    let app = server.router();
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/cloud/enroll")
                .header("X-FamShield-Token", &token)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"enrollment_code":"654321"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let app = server.router();
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/cloud/disenroll")
                .header("X-FamShield-Token", &token)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_centralized_server_settings_lifecycle() {
    use axum::body::Body;
    use axum::http::{header, Request, StatusCode};
    use famshield_local_admin::LocalAdminServer;
    use std::sync::Arc;
    use tower::ServiceExt;

    let storage = StorageEngine::in_memory().unwrap();
    let service = Arc::new(LocalAdminService::new(storage.clone()));
    let server = LocalAdminServer::new(service.clone(), None);

    // 1. Initial state: StandaloneFree, default URL
    let cfg = service.get_server_config().unwrap();
    assert_eq!(cfg.operating_mode, "StandaloneFree");
    assert_eq!(cfg.server_url, "https://api.famshield.cloud");
    assert_eq!(cfg.sync_interval_secs, 60);
    assert!(cfg.tls_verify);
    assert_eq!(cfg.connection_status, "Disconnected");

    // 2. Authenticated request to update server config
    let token = server.create_session_token().await;
    let app = server.router();
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/settings/server")
                .header("X-FamShield-Token", &token)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    r#"{
                        "server_url": "https://famshield.mycorp.lan:8443",
                        "tenant_id": "corp-dept-1",
                        "device_name": "PC-KID-01",
                        "enrollment_code": "884422",
                        "sync_interval_secs": 30,
                        "tls_verify": false
                    }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let updated_cfg = service.get_server_config().unwrap();
    assert_eq!(updated_cfg.operating_mode, "OnlineManaged");
    assert_eq!(updated_cfg.server_url, "https://famshield.mycorp.lan:8443");
    assert_eq!(updated_cfg.tenant_id, "corp-dept-1");
    assert_eq!(updated_cfg.device_name, "PC-KID-01");
    assert_eq!(updated_cfg.device_id, "dev-884422");
    assert_eq!(updated_cfg.sync_interval_secs, 30);
    assert!(!updated_cfg.tls_verify);
    assert_eq!(updated_cfg.connection_status, "Connected");

    // 3. Test connection probe
    let app = server.router();
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/settings/server/test")
                .header("X-FamShield-Token", &token)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    r#"{"server_url":"https://api.famshield.cloud"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 4. Test sync trigger
    let app = server.router();
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/settings/server/sync")
                .header("X-FamShield-Token", &token)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let synced_cfg = service.get_server_config().unwrap();
    assert!(synced_cfg.last_sync_time.is_some());
    assert_eq!(synced_cfg.policy_version, 2);

    // 5. Revert to StandaloneFree
    let app = server.router();
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/settings/server")
                .header("X-FamShield-Token", &token)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"operating_mode":"StandaloneFree"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let reverted_cfg = service.get_server_config().unwrap();
    assert_eq!(reverted_cfg.operating_mode, "StandaloneFree");
    assert_eq!(reverted_cfg.connection_status, "Disconnected");
}

#[tokio::test]
async fn test_bulk_rules_import_and_sync() {
    use axum::body::Body;
    use axum::http::{header, Request, StatusCode};
    use famshield_dns::blocklist::BlocklistStore;
    use famshield_local_admin::LocalAdminServer;
    use std::sync::Arc;
    use tower::ServiceExt;

    let storage = StorageEngine::in_memory().unwrap();
    let service = Arc::new(LocalAdminService::new(storage.clone()));
    let dns_blocklist = Arc::new(BlocklistStore::new());
    let server = LocalAdminServer::new(service.clone(), Some(dns_blocklist.clone()));
    let token = server.create_session_token().await;

    let app = server.router();
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/rules/bulk")
                .header("X-FamShield-Token", &token)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    r#"[
                        {"rule_id":"gambling.org","rule_type":"domain","payload_json":"{\"action\":\"BLOCK\"}","enabled":true},
                        {"rule_id":"badgame.com","rule_type":"domain","payload_json":"{\"action\":\"BLOCK\"}","enabled":true},
                        {"rule_id":"game.exe","rule_type":"application","payload_json":"{\"max_minutes_per_day\":45}","enabled":true}
                    ]"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CREATED);
    assert_eq!(service.list_rules().unwrap().len(), 3);
    assert!(dns_blocklist.is_blocked("gambling.org"));
    assert!(dns_blocklist.is_blocked("badgame.com"));
    assert!(!dns_blocklist.is_blocked("safe.org"));
}

#[tokio::test]
async fn test_monitor_mode_toggle_and_dns_non_blocking() {
    use famshield_dns::blocklist::BlocklistStore;
    use famshield_local_admin::LocalAdminServer;
    use std::sync::Arc;

    let storage = StorageEngine::in_memory().unwrap();
    let service = Arc::new(LocalAdminService::new(storage.clone()));
    let dns_blocklist = Arc::new(BlocklistStore::new());
    dns_blocklist.replace_blocklist(vec!["malicious-test.com"]);

    let _server = LocalAdminServer::new(service.clone(), Some(dns_blocklist.clone()));

    // Default: monitor_mode is false, blocking is active
    assert!(!service.is_monitor_mode());
    assert!(dns_blocklist.should_block("malicious-test.com"));

    // Enable monitor mode via service
    service.set_monitor_mode(true).unwrap();
    assert!(service.is_monitor_mode());

    // Update DNS blocklist with monitor mode
    dns_blocklist.set_monitor_mode(true);
    assert!(dns_blocklist.is_monitor_mode());
    // In monitor mode, should_block returns false even for listed domains
    assert!(!dns_blocklist.should_block("malicious-test.com"));
    // But is_blocked still detects it
    assert!(dns_blocklist.is_blocked("malicious-test.com"));

    // Toggle back to active blocking
    service.set_monitor_mode(false).unwrap();
    dns_blocklist.set_monitor_mode(false);
    assert!(dns_blocklist.should_block("malicious-test.com"));
}

#[tokio::test]
async fn test_backup_and_restore_lifecycle_with_checksum_validation() {
    use famshield_local_admin::service::CreateRuleRequest;

    let storage = StorageEngine::in_memory().unwrap();
    let service = LocalAdminService::new(storage.clone());

    // Create initial rule
    service
        .create_rule(CreateRuleRequest {
            rule_id: "blocked-gaming.com".to_string(),
            rule_type: "domain".to_string(),
            payload_json: r#"{"action":"BLOCK"}"#.to_string(),
            enabled: true,
        })
        .unwrap();

    service.set_monitor_mode(true).unwrap();

    // Create backup
    let backup = service.create_backup().unwrap();
    assert_eq!(backup.format_version, "1.0");
    assert!(backup.verify_checksum());
    assert_eq!(backup.rules.len(), 1);
    assert!(backup.settings.monitor_mode);

    // Tampered backup should fail checksum verification
    let mut tampered = backup.clone();
    tampered.checksum =
        "0000000000000000000000000000000000000000000000000000000000000000".to_string();
    assert!(!tampered.verify_checksum());

    let restore_err = service.restore_backup(tampered);
    assert!(restore_err.is_err());

    // Reset storage to fresh in-memory instance
    let fresh_storage = StorageEngine::in_memory().unwrap();
    let fresh_service = LocalAdminService::new(fresh_storage.clone());
    assert_eq!(fresh_service.list_rules().unwrap().len(), 0);
    assert!(!fresh_service.is_monitor_mode());

    // Restore legitimate backup
    let result = fresh_service.restore_backup(backup).unwrap();
    assert!(result.success);
    assert_eq!(result.rules_restored, 1);
    assert!(fresh_service.is_monitor_mode());
    assert_eq!(fresh_service.list_rules().unwrap().len(), 1);
    assert_eq!(
        fresh_service.list_rules().unwrap()[0].rule_id,
        "blocked-gaming.com"
    );
}

#[tokio::test]
async fn test_api_protection_mode_and_backup_endpoints() {
    use axum::body::Body;
    use axum::http::{header, Request, StatusCode};
    use famshield_dns::blocklist::BlocklistStore;
    use famshield_local_admin::LocalAdminServer;
    use http_body_util::BodyExt;
    use std::sync::Arc;
    use tower::ServiceExt;

    let storage = StorageEngine::in_memory().unwrap();
    let service = Arc::new(LocalAdminService::new(storage.clone()));
    let dns_blocklist = Arc::new(BlocklistStore::new());
    let server = LocalAdminServer::new(service.clone(), Some(dns_blocklist.clone()));
    let token = server.create_session_token().await;

    // 1. GET /api/settings/protection-mode
    let res = server
        .router()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/settings/protection-mode")
                .header("X-FamShield-Token", &token)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 2. POST /api/settings/protection-mode (enable monitor mode)
    let res = server
        .router()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/settings/protection-mode")
                .header("X-FamShield-Token", &token)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"monitor_mode": true}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert!(service.is_monitor_mode());
    assert!(dns_blocklist.is_monitor_mode());

    // 3. GET /api/backup/export
    let res = server
        .router()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/backup/export")
                .header("X-FamShield-Token", &token)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body_bytes = res.into_body().collect().await.unwrap().to_bytes();
    let backup_json_str = std::str::from_utf8(&body_bytes).unwrap();
    assert!(backup_json_str.contains("\"format_version\": \"1.0\""));
    assert!(backup_json_str.contains("\"monitor_mode\": true"));

    // 4. POST /api/backup/restore
    let res = server
        .router()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/backup/restore")
                .header("X-FamShield-Token", &token)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(backup_json_str.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_activity_summary_endpoint_and_usage_tracking() {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use famshield_dns::blocklist::BlocklistStore;
    use famshield_local_admin::LocalAdminServer;
    use famshield_storage::UsageSegmentRecord;
    use http_body_util::BodyExt;
    use std::sync::Arc;
    use tower::ServiceExt;

    let storage = StorageEngine::in_memory().unwrap();
    let service = Arc::new(LocalAdminService::new(storage.clone()));
    let dns_blocklist = Arc::new(BlocklistStore::new());
    let server = LocalAdminServer::new(service.clone(), Some(dns_blocklist.clone()));
    let token = server.create_session_token().await;

    // Record an app usage segment in storage
    let seg = UsageSegmentRecord {
        segment_id: "test-seg-1".to_string(),
        child_id: "local_child".to_string(),
        rule_id: "RobloxPlayerBeta.exe".to_string(),
        start_utc: Utc::now() - chrono::Duration::minutes(30),
        duration_ms: 1_800_000,
        state: "committed".to_string(),
    };
    storage.record_usage_segment(&seg).unwrap();

    // Call GET /api/activity/summary
    let res = server
        .router()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/activity/summary")
                .header("X-FamShield-Token", &token)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body_bytes = res.into_body().collect().await.unwrap().to_bytes();
    let summary_json = std::str::from_utf8(&body_bytes).unwrap();
    assert!(summary_json.contains("total_screen_time_minutes"));
    assert!(summary_json.contains("app_activities"));
    assert!(summary_json.contains("web_activities"));
    assert!(summary_json.contains("unified_timeline"));
    assert!(summary_json.contains("Roblox"));
}

#[tokio::test]
async fn test_keyword_monitoring_api_and_violation_flow() {
    use axum::body::Body;
    use axum::http::{header, Request, StatusCode};
    use famshield_dns::blocklist::BlocklistStore;
    use famshield_local_admin::LocalAdminServer;
    use http_body_util::BodyExt;
    use serde_json::Value;
    use std::sync::Arc;
    use tower::ServiceExt;

    let storage = StorageEngine::in_memory().unwrap();
    let service = Arc::new(LocalAdminService::new(storage.clone()));
    let dns_blocklist = Arc::new(BlocklistStore::new());
    let server = LocalAdminServer::new(service.clone(), Some(dns_blocklist.clone()));
    let token = server.create_session_token().await;

    // 1. GET /api/keywords
    let res = server
        .router()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/keywords")
                .header("X-FamShield-Token", &token)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let config: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(config["keyboard_monitoring_enabled"], true);
    assert!(config["rules"].as_array().unwrap().len() >= 20);

    // 2. POST /api/keywords (Add custom keyword)
    let res = server
        .router()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/keywords")
                .header("X-FamShield-Token", &token)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    r#"{"pattern":"tụ tập đánh nhau","category":"Bắt nạt / Đe dọa","severity":"high"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let added_rule: Value = serde_json::from_slice(&bytes).unwrap();
    let rule_id = added_rule["id"].as_str().unwrap().to_string();
    assert_eq!(added_rule["pattern"], "tụ tập đánh nhau");
    assert_eq!(added_rule["is_custom"], true);

    // 3. POST /api/keywords/toggle (Disable custom keyword)
    let res = server
        .router()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/keywords/toggle")
                .header("X-FamShield-Token", &token)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(format!(
                    r#"{{"id":"{}","enabled":false}}"#,
                    rule_id
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 4. POST /api/keywords/violation rejects unauthenticated injection.
    let res = server
        .router()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/keywords/violation")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    r#"{"app_name":"game.exe","keyword":"vcl","category":"Chửi bậy / Thô tục","severity":"medium"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    // 5. An authenticated violation event is persisted.
    let res = server
        .router()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/keywords/violation")
                    .header("X-FamShield-Token", &token)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    r#"{"app_name":"game.exe","keyword":"vcl","category":"Chửi bậy / Thô tục","severity":"medium"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    // 6. GET /api/activity/summary should now include this keyword violation
    let res = server
        .router()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/activity/summary")
                .header("X-FamShield-Token", &token)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let summary: Value = serde_json::from_slice(&bytes).unwrap();
    assert!(!summary["keyword_violations"].as_array().unwrap().is_empty());
    assert!(summary["unified_timeline"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["item_type"] == "keyword"));

    // 7. DELETE /api/keywords/:id
    let res = server
        .router()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/keywords/{}", rule_id))
                .header("X-FamShield-Token", &token)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_sync_rules_to_cloud_endpoint() {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use famshield_local_admin::LocalAdminServer;
    use std::sync::Arc;
    use tower::ServiceExt;

    let storage = StorageEngine::in_memory().unwrap();
    let service = Arc::new(LocalAdminService::new(storage.clone()));
    let server = LocalAdminServer::new(service, None);
    let token = server.create_session_token().await;

    // 1. Without server_url configured, should return 400 Bad Request
    let res = server
        .router()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/rules/sync-to-cloud")
                .header("X-FamShield-Token", &token)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    // 2. Set server_url and add a local domain rule
    storage
        .set_meta("server_url", "http://127.0.0.1:54321")
        .unwrap();
    storage
        .set_meta("enrolled_tenant_id", "test-tenant")
        .unwrap();
    storage
        .upsert_local_rule(&famshield_storage::LocalRuleRecord {
            rule_id: "tiktok.com".to_string(),
            rule_type: "domain".to_string(),
            payload: r#"{"action":"BLOCK"}"#.to_string(),
            enabled: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
        .unwrap();

    // With unreachable server, returns 502 Bad Gateway
    let res = server
        .router()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/rules/sync-to-cloud")
                .header("X-FamShield-Token", &token)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_GATEWAY);
}

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

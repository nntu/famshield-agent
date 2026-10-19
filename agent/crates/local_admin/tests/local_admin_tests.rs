use famshield_local_admin::{
    auth::MasterPinAuthenticator,
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
    let service = LocalAdminService::new(storage);

    // 1. Initial default PIN is "1234"
    assert!(service.authenticate_storage("1234").unwrap());
    assert!(!service.authenticate_storage("9999").unwrap());

    // 2. Set new PIN "5678" -> Generates Recovery Key
    let recovery_key = service.set_master_pin("5678").unwrap();
    assert!(recovery_key.starts_with("FAM-"));
    assert_eq!(recovery_key.len(), 18); // "FAM-XXXX-XXXX-XXXX"

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


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

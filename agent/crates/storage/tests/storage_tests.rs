use chrono::Utc;
use famshield_core::policy::Policy;
use famshield_storage::{LocalRuleRecord, QueryLogRecord, StorageEngine, VisualEvidenceRecord};

const SAMPLE_POLICY: &str = include_str!("../../../../contracts/examples/policy.v1.json");

#[test]
fn test_storage_policy_lifecycle() {
    let storage = StorageEngine::in_memory().unwrap();
    let policy: Policy = serde_json::from_str(SAMPLE_POLICY).unwrap();

    storage.save_policy(&policy, SAMPLE_POLICY).unwrap();
    let loaded = storage
        .load_active_policy()
        .unwrap()
        .expect("Policy should be loaded");

    assert_eq!(loaded.policy_id, policy.policy_id);
    assert_eq!(loaded.version, policy.version);
}

#[test]
fn test_storage_local_rules_crud() {
    let storage = StorageEngine::in_memory().unwrap();

    let rule = LocalRuleRecord {
        rule_id: "local-app-block-1".into(),
        rule_type: "application".into(),
        payload: "{}".into(),
        enabled: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    storage.upsert_local_rule(&rule).unwrap();

    let rules = storage.list_local_rules().unwrap();
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].rule_id, "local-app-block-1");

    let deleted = storage.delete_local_rule("local-app-block-1").unwrap();
    assert!(deleted);
    let rules_after = storage.list_local_rules().unwrap();
    assert_eq!(rules_after.len(), 0);
}

#[test]
fn test_query_logs_batch_insert() {
    let storage = StorageEngine::in_memory().unwrap();

    let records = vec![
        QueryLogRecord {
            domain: "example.com".into(),
            qtype: "A".into(),
            action: "allow".into(),
            reason: "default_action_allow".into(),
            rule_id: None,
            occurred_at: Utc::now(),
        },
        QueryLogRecord {
            domain: "ads.track.invalid".into(),
            qtype: "AAAA".into(),
            action: "block".into(),
            reason: "explicit_rule_block".into(),
            rule_id: Some("block-ads".into()),
            occurred_at: Utc::now(),
        },
    ];

    storage.insert_query_logs_batch(&records).unwrap();
}

#[test]
fn test_visual_evidence_purge() {
    let storage = StorageEngine::in_memory().unwrap();
    let now = Utc::now();

    // Expired record
    let expired = VisualEvidenceRecord {
        evidence_id: "ev-expired-1".into(),
        event_id: "evt-1".into(),
        occurred_at: now - chrono::Duration::hours(50),
        trigger: "app_blocked".into(),
        image_payload: vec![1, 2, 3],
        size_bytes: 3,
        expires_at: now - chrono::Duration::hours(2),
    };
    storage.save_visual_evidence(&expired).unwrap();

    // Active record
    let active = VisualEvidenceRecord {
        evidence_id: "ev-active-2".into(),
        event_id: "evt-2".into(),
        occurred_at: now,
        trigger: "domain_blocked".into(),
        image_payload: vec![4, 5, 6],
        size_bytes: 3,
        expires_at: now + chrono::Duration::hours(46),
    };
    storage.save_visual_evidence(&active).unwrap();

    let purged = storage.purge_expired_evidence(now).unwrap();
    assert_eq!(purged, 1);
}

#[test]
fn test_error_logs_smart_deduplication() {
    let storage = StorageEngine::in_memory().unwrap();

    // 1. Record same error multiple times
    let id1 = storage
        .record_error_log("dns_proxy", "ERROR", "Upstream DNS timeout: 1.1.1.1:53", Some("stack1"))
        .unwrap();
    let id2 = storage
        .record_error_log("dns_proxy", "ERROR", "Upstream DNS timeout: 1.1.1.1:53", Some("stack2"))
        .unwrap();
    let id3 = storage
        .record_error_log("dns_proxy", "ERROR", "Upstream DNS timeout: 1.1.1.1:53", None)
        .unwrap();

    // Fingerprint ID must be identical for identical error pattern
    assert_eq!(id1, id2);
    assert_eq!(id2, id3);

    // 2. Query unsynced errors: should only have 1 deduplicated record with count = 3
    let errors = storage.list_unsynced_errors(10).unwrap();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].error_id, id1);
    assert_eq!(errors[0].occurrence_count, 3);
    assert_eq!(errors[0].module, "dns_proxy");

    // 3. Mark as synced
    storage.mark_errors_synced(std::slice::from_ref(&id1)).unwrap();
    let unsynced_after = storage.list_unsynced_errors(10).unwrap();
    assert_eq!(unsynced_after.len(), 0);

    // 4. If same error happens again, it updates count to 4 and marks synced = 0 again
    let id4 = storage
        .record_error_log("dns_proxy", "ERROR", "Upstream DNS timeout: 1.1.1.1:53", None)
        .unwrap();
    assert_eq!(id4, id1);

    let re_emerged = storage.list_unsynced_errors(10).unwrap();
    assert_eq!(re_emerged.len(), 1);
    assert_eq!(re_emerged[0].occurrence_count, 4);
    assert!(!re_emerged[0].synced);
}

#[test]
fn test_outbox_lifecycle_and_retry_backoff() {
    use chrono::Duration;

    let storage = StorageEngine::in_memory().unwrap();
    let t0 = Utc::now();

    // 1. Enqueue two events
    storage
        .enqueue_outbox("evt-01", "heartbeat", t0, r#"{"uptime": 100}"#)
        .unwrap();
    storage
        .enqueue_outbox("evt-02", "dns_query", t0 + Duration::seconds(1), r#"{"domain": "safe.org"}"#)
        .unwrap();

    assert_eq!(storage.count_outbox().unwrap(), 2);

    // 2. Peek outbox
    let pending = storage.peek_outbox_at(10, t0 + Duration::seconds(2)).unwrap();
    assert_eq!(pending.len(), 2);
    assert_eq!(pending[0].event_id, "evt-01");
    assert_eq!(pending[0].attempts, 0);
    assert_eq!(pending[1].event_id, "evt-02");

    // 3. Fail evt-01 with backoff 30s
    storage
        .fail_outbox_events(&["evt-01".to_string()], t0 + Duration::seconds(30))
        .unwrap();

    // At t0 + 10s: only evt-02 is ready, evt-01 is backing off
    let ready_t10 = storage.peek_outbox_at(10, t0 + Duration::seconds(10)).unwrap();
    assert_eq!(ready_t10.len(), 1);
    assert_eq!(ready_t10[0].event_id, "evt-02");

    // Dequeue evt-02 on success
    storage.dequeue_outbox(&["evt-02".to_string()]).unwrap();
    assert_eq!(storage.count_outbox().unwrap(), 1);

    // At t0 + 35s: evt-01 becomes ready again with attempt count = 1
    let ready_t35 = storage.peek_outbox_at(10, t0 + Duration::seconds(35)).unwrap();
    assert_eq!(ready_t35.len(), 1);
    assert_eq!(ready_t35[0].event_id, "evt-01");
    assert_eq!(ready_t35[0].attempts, 1);

    // Dequeue evt-01 -> outbox is empty
    storage.dequeue_outbox(&["evt-01".to_string()]).unwrap();
    assert_eq!(storage.count_outbox().unwrap(), 0);
}


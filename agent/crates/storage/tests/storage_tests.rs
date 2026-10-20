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
        .record_error_log(
            "dns_proxy",
            "ERROR",
            "Upstream DNS timeout: 1.1.1.1:53",
            Some("stack1"),
        )
        .unwrap();
    let id2 = storage
        .record_error_log(
            "dns_proxy",
            "ERROR",
            "Upstream DNS timeout: 1.1.1.1:53",
            Some("stack2"),
        )
        .unwrap();
    let id3 = storage
        .record_error_log(
            "dns_proxy",
            "ERROR",
            "Upstream DNS timeout: 1.1.1.1:53",
            None,
        )
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
    storage
        .mark_errors_synced(std::slice::from_ref(&id1))
        .unwrap();
    let unsynced_after = storage.list_unsynced_errors(10).unwrap();
    assert_eq!(unsynced_after.len(), 0);

    // 4. If same error happens again, it updates count to 4 and marks synced = 0 again
    let id4 = storage
        .record_error_log(
            "dns_proxy",
            "ERROR",
            "Upstream DNS timeout: 1.1.1.1:53",
            None,
        )
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
        .enqueue_outbox(
            "evt-02",
            "dns_query",
            t0 + Duration::seconds(1),
            r#"{"domain": "safe.org"}"#,
        )
        .unwrap();

    assert_eq!(storage.count_outbox().unwrap(), 2);

    // 2. Peek outbox
    let pending = storage
        .peek_outbox_at(10, t0 + Duration::seconds(2))
        .unwrap();
    assert_eq!(pending.len(), 2);
    assert_eq!(pending[0].event_id, "evt-01");
    assert_eq!(pending[0].attempts, 0);
    assert_eq!(pending[1].event_id, "evt-02");

    // 3. Fail evt-01 with backoff 30s
    storage
        .fail_outbox_events(&["evt-01".to_string()], t0 + Duration::seconds(30))
        .unwrap();

    // At t0 + 10s: only evt-02 is ready, evt-01 is backing off
    let ready_t10 = storage
        .peek_outbox_at(10, t0 + Duration::seconds(10))
        .unwrap();
    assert_eq!(ready_t10.len(), 1);
    assert_eq!(ready_t10[0].event_id, "evt-02");

    // Dequeue evt-02 on success
    storage.dequeue_outbox(&["evt-02".to_string()]).unwrap();
    assert_eq!(storage.count_outbox().unwrap(), 1);

    // At t0 + 35s: evt-01 becomes ready again with attempt count = 1
    let ready_t35 = storage
        .peek_outbox_at(10, t0 + Duration::seconds(35))
        .unwrap();
    assert_eq!(ready_t35.len(), 1);
    assert_eq!(ready_t35[0].event_id, "evt-01");
    assert_eq!(ready_t35[0].attempts, 1);

    // Dequeue evt-01 -> outbox is empty
    storage.dequeue_outbox(&["evt-01".to_string()]).unwrap();
    assert_eq!(storage.count_outbox().unwrap(), 0);
}

#[test]
fn test_domain_query_summaries_and_drilldown() {
    let storage = StorageEngine::in_memory().unwrap();
    let now = Utc::now();

    let logs = vec![
        QueryLogRecord {
            domain: "tiktok.com".into(),
            qtype: "A".into(),
            action: "BLOCK".into(),
            reason: "local_domain_rule".into(),
            rule_id: Some("tiktok.com".into()),
            occurred_at: now - chrono::Duration::seconds(10),
        },
        QueryLogRecord {
            domain: "tiktok.com".into(),
            qtype: "AAAA".into(),
            action: "BLOCK".into(),
            reason: "local_domain_rule".into(),
            rule_id: Some("tiktok.com".into()),
            occurred_at: now - chrono::Duration::seconds(5),
        },
        QueryLogRecord {
            domain: "google.com".into(),
            qtype: "A".into(),
            action: "ALLOW".into(),
            reason: "upstream".into(),
            rule_id: None,
            occurred_at: now,
        },
    ];

    storage.insert_query_logs_batch(&logs).unwrap();

    let summaries = storage.get_domain_query_summaries(10).unwrap();
    assert_eq!(summaries.len(), 2);
    // tiktok.com has 2 queries, should be first
    assert_eq!(summaries[0].domain, "tiktok.com");
    assert_eq!(summaries[0].total_queries, 2);
    assert_eq!(summaries[0].blocked_count, 2);
    assert_eq!(summaries[0].allowed_count, 0);

    // google.com has 1 query
    assert_eq!(summaries[1].domain, "google.com");
    assert_eq!(summaries[1].total_queries, 1);
    assert_eq!(summaries[1].allowed_count, 1);
    assert_eq!(summaries[1].blocked_count, 0);

    // Drill-down for tiktok.com
    let tiktok_logs = storage
        .list_query_logs_for_domain("tiktok.com", 10)
        .unwrap();
    assert_eq!(tiktok_logs.len(), 2);
    assert_eq!(tiktok_logs[0].domain, "tiktok.com");
    assert_eq!(tiktok_logs[1].domain, "tiktok.com");
}

#[test]
fn test_rollup_and_prune_query_logs() {
    let storage = StorageEngine::in_memory().unwrap();
    let now = Utc::now();
    let ten_days_ago = now - chrono::Duration::days(10);
    let one_hour_ago = now - chrono::Duration::hours(1);

    // Old logs (10 days ago): 3 queries for youtube.com (2 ALLOW, 1 BLOCK) and 1 for safe.org
    let old_logs = vec![
        QueryLogRecord {
            domain: "youtube.com".into(),
            qtype: "A".into(),
            action: "ALLOW".into(),
            reason: "upstream".into(),
            rule_id: None,
            occurred_at: ten_days_ago,
        },
        QueryLogRecord {
            domain: "youtube.com".into(),
            qtype: "AAAA".into(),
            action: "ALLOW".into(),
            reason: "upstream".into(),
            rule_id: None,
            occurred_at: ten_days_ago + chrono::Duration::seconds(1),
        },
        QueryLogRecord {
            domain: "youtube.com".into(),
            qtype: "A".into(),
            action: "BLOCK".into(),
            reason: "quota".into(),
            rule_id: Some("youtube.com".into()),
            occurred_at: ten_days_ago + chrono::Duration::seconds(2),
        },
        QueryLogRecord {
            domain: "safe.org".into(),
            qtype: "A".into(),
            action: "ALLOW".into(),
            reason: "upstream".into(),
            rule_id: None,
            occurred_at: ten_days_ago,
        },
    ];
    storage.insert_query_logs_batch(&old_logs).unwrap();

    // Recent logs (1 hour ago): 2 queries for youtube.com (both ALLOW)
    let recent_logs = vec![
        QueryLogRecord {
            domain: "youtube.com".into(),
            qtype: "A".into(),
            action: "ALLOW".into(),
            reason: "upstream".into(),
            rule_id: None,
            occurred_at: one_hour_ago,
        },
        QueryLogRecord {
            domain: "youtube.com".into(),
            qtype: "AAAA".into(),
            action: "ALLOW".into(),
            reason: "upstream".into(),
            rule_id: None,
            occurred_at: one_hour_ago + chrono::Duration::seconds(1),
        },
    ];
    storage.insert_query_logs_batch(&recent_logs).unwrap();

    let (raw_before, agg_before) = storage.get_query_logs_storage_stats().unwrap();
    assert_eq!(raw_before, 6);
    assert_eq!(agg_before, 0);

    // Prune logs older than 7 days
    let cutoff = now - chrono::Duration::days(7);
    let deleted = storage.rollup_and_prune_query_logs(cutoff).unwrap();
    assert_eq!(deleted, 4);

    let (raw_after, agg_after) = storage.get_query_logs_storage_stats().unwrap();
    assert_eq!(raw_after, 2); // only 2 recent queries remain in raw table
    assert_eq!(agg_after, 2); // 2 domains in aggregates table: youtube.com, safe.org

    // CRITICAL: get_domain_query_summaries must seamlessly combine aggregates + recent raw logs!
    let summaries = storage.get_domain_query_summaries(10).unwrap();
    assert_eq!(summaries.len(), 2);

    // youtube.com should have TOTAL 5 queries (3 old + 2 recent), 4 ALLOW, 1 BLOCK
    let yt = summaries.iter().find(|s| s.domain == "youtube.com").unwrap();
    assert_eq!(yt.total_queries, 5);
    assert_eq!(yt.allowed_count, 4);
    assert_eq!(yt.blocked_count, 1);

    // safe.org has 1 query (from old aggregate)
    let safe = summaries.iter().find(|s| s.domain == "safe.org").unwrap();
    assert_eq!(safe.total_queries, 1);
    assert_eq!(safe.allowed_count, 1);
    assert_eq!(safe.blocked_count, 0);
}

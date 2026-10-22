use chrono::Utc;
use famshield_storage::StorageEngine;
use famshield_sync::{
    AppUsagePayload, DnsQueryPayload, HeartbeatPayload, SecurityAlertPayload, TelemetryEnvelope,
    TelemetryEvent, TelemetryEventType, TelemetryPipeline, CURRENT_TELEMETRY_VERSION,
};

#[test]
fn test_telemetry_envelope_serialization_roundtrip() {
    let now = Utc::now();
    let events = vec![
        TelemetryEvent::new(
            "evt-01",
            TelemetryEventType::Heartbeat.as_str(),
            now,
            serde_json::to_value(HeartbeatPayload {
                uptime_seconds: 7200,
                agent_version: "0.1.0".to_string(),
                wfp_active: true,
                dns_active: true,
                pending_outbox_count: 0,
                status: "HEALTHY".to_string(),
                dns_upstream_optimal: Some("1.1.1.1".to_string()),
                dns_avg_rtt_ms: Some(12),
            })
            .unwrap(),
        ),
        TelemetryEvent::new(
            "evt-02",
            TelemetryEventType::SecurityAlert.as_str(),
            now,
            serde_json::to_value(SecurityAlertPayload {
                alert_type: "pin_lockout".to_string(),
                severity: "HIGH".to_string(),
                description: "Master PIN lockout triggered after 5 failures".to_string(),
                metadata: serde_json::json!({ "failed_attempts": 5 }),
            })
            .unwrap(),
        ),
        TelemetryEvent::new(
            "evt-03",
            TelemetryEventType::DnsQuery.as_str(),
            now,
            serde_json::to_value(DnsQueryPayload {
                domain: "phishing-bank.xyz".to_string(),
                qtype: "A".to_string(),
                action: "block".to_string(),
                reason: "blocklist_match".to_string(),
                rule_id: Some("rule-malware-01".to_string()),
                latency_ms: Some(15),
                upstream: Some("1.1.1.1".to_string()),
            })
            .unwrap(),
        ),
        TelemetryEvent::new(
            "evt-04",
            TelemetryEventType::AppUsage.as_str(),
            now,
            serde_json::to_value(AppUsagePayload {
                app_name: "chrome.exe".to_string(),
                duration_ms: 120_000,
                session_count: 3,
                start_time: now - chrono::Duration::minutes(10),
                end_time: now,
                category: Some("browsers".to_string()),
            })
            .unwrap(),
        ),
    ];

    let envelope = TelemetryEnvelope::new("dev-pc-01", events);
    assert_eq!(envelope.version, CURRENT_TELEMETRY_VERSION);
    assert_eq!(envelope.version, "1.1");
    assert_eq!(envelope.device_id, "dev-pc-01");
    assert_eq!(envelope.event_count, 4);
    assert!(envelope.batch_id.starts_with("batch-dev-pc-01-"));

    let json = envelope.to_json().unwrap();
    let deserialized: TelemetryEnvelope = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, envelope);
}

#[test]
fn test_telemetry_pipeline_outbox_queue_and_batching() {
    let storage = StorageEngine::in_memory().unwrap();
    let pipeline = TelemetryPipeline::new("dev-pc-99", storage);

    // 1. Initial pending count is 0
    assert_eq!(pipeline.pending_count().unwrap(), 0);
    assert!(pipeline.prepare_next_batch(10).unwrap().is_none());

    // 2. Queue multiple events
    let hb_id = pipeline.queue_heartbeat(3600, "0.1.0", true, true).unwrap();
    assert!(hb_id.starts_with("hb-dev-pc-99-"));

    let now = Utc::now();
    let dns_id = pipeline
        .queue_dns_query_detailed(
            "example.com",
            "A",
            "allow",
            "no_match",
            None,
            Some(8),
            Some("1.1.1.1"),
            now,
        )
        .unwrap();
    assert!(dns_id.starts_with("dns-dev-pc-99-"));

    let sec_id = pipeline
        .queue_security_alert(
            "tamper_attempt",
            "CRITICAL",
            "hosts file modified",
            serde_json::json!({}),
        )
        .unwrap();
    assert!(sec_id.starts_with("sec-dev-pc-99-"));

    let app_id = pipeline
        .queue_app_usage(
            "code.exe",
            180_000,
            2,
            now - chrono::Duration::minutes(15),
            now,
            Some("development"),
        )
        .unwrap();
    assert!(app_id.starts_with("app-dev-pc-99-"));

    assert_eq!(pipeline.pending_count().unwrap(), 4);

    // 3. Prepare next batch with limit 2
    let batch1 = pipeline
        .prepare_next_batch(2)
        .unwrap()
        .expect("Batch should exist");
    assert_eq!(batch1.event_count, 2);
    assert_eq!(batch1.device_id, "dev-pc-99");
    assert_eq!(batch1.events[0].event_id, hb_id);
    assert_eq!(batch1.events[1].event_id, dns_id);

    // 4. Acknowledge batch1 -> 2 events remain
    pipeline.acknowledge_batch(&batch1).unwrap();
    assert_eq!(pipeline.pending_count().unwrap(), 2);

    // 5. Prepare next batch -> gets the remaining security alert & app usage
    let batch2 = pipeline
        .prepare_next_batch(2)
        .unwrap()
        .expect("Batch should exist");
    assert_eq!(batch2.event_count, 2);
    assert_eq!(batch2.events[0].event_id, sec_id);
    assert_eq!(batch2.events[1].event_id, app_id);

    // 6. Simulate transmission failure on batch2 with retry backoff
    pipeline.report_batch_failure(&batch2, 60).unwrap();

    // Because it's backing off, next batch peek right now will find nothing ready
    let nothing_ready = pipeline.prepare_next_batch(10).unwrap();
    assert!(nothing_ready.is_none());

    // Still in outbox awaiting retry
    assert_eq!(pipeline.pending_count().unwrap(), 2);

    // Acknowledge after successful retry
    pipeline.acknowledge_batch(&batch2).unwrap();
    assert_eq!(pipeline.pending_count().unwrap(), 0);
}

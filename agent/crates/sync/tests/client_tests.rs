use famshield_dns::blocklist::BlocklistStore;
use famshield_storage::StorageEngine;
use famshield_sync::{
    BatchEventsResponse, CloudSyncClient, HeartbeatRequest, HeartbeatResponse, TelemetryPipeline,
    TelemetrySyncWorker, WebFilterPolicyDto,
};
use std::sync::Arc;

#[test]
fn test_heartbeat_payload_serialization_and_deserialization() {
    let req = HeartbeatRequest {
        device_id: "dev-win11-test".to_string(),
        agent_version: "0.1.1".to_string(),
        policy_version: 2,
        health_status: Some("healthy".to_string()),
        status: Some("healthy".to_string()),
        uptime_seconds: 7200,
        wfp_active: true,
        dns_active: true,
        outbox_depth: 4,
        dns_upstream_optimal: Some("1.1.1.1".to_string()),
        dns_avg_rtt_ms: Some(15),
    };

    let json = serde_json::to_string(&req).expect("Failed to serialize HeartbeatRequest");
    assert!(json.contains("\"device_id\":\"dev-win11-test\""));
    assert!(json.contains("\"uptime_seconds\":7200"));
    assert!(json.contains("\"outbox_depth\":4"));

    let resp_json = r#"{
        "server_time": "2026-10-19T16:00:00Z",
        "desired_policy_version": 3,
        "command_wait_seconds": 25,
        "available_release": {
            "manifest_version": 1,
            "version": "0.1.2",
            "channel": "stable",
            "min_supported_version": "0.1.0",
            "service_sha256": "abcdef1234567890",
            "service_binary_url": "https://dl.famshield.cloud/releases/v0.1.2/famshield-service.exe",
            "signature_hex": "3045022100..."
        }
    }"#;

    let resp: HeartbeatResponse =
        serde_json::from_str(resp_json).expect("Failed to deserialize HeartbeatResponse");
    assert_eq!(resp.desired_policy_version, 3);
    assert_eq!(resp.command_wait_seconds, 25);
    assert_eq!(resp.server_time, "2026-10-19T16:00:00Z");

    let rel = resp.available_release.expect("Expected available release");
    assert_eq!(rel.version, "0.1.2");
    assert_eq!(rel.channel, "stable");
}

#[test]
fn test_batch_events_response_deserialization() {
    let json = r#"{
        "batch_id": "batch-1234-abcd",
        "accepted": ["evt-01", "evt-02"],
        "duplicate": ["evt-03"]
    }"#;

    let resp: BatchEventsResponse =
        serde_json::from_str(json).expect("Failed to deserialize BatchEventsResponse");
    assert_eq!(resp.batch_id, "batch-1234-abcd");
    assert_eq!(resp.accepted.len(), 2);
    assert_eq!(resp.duplicate.len(), 1);
}

#[test]
fn test_web_filter_policy_deserialization() {
    let json = r#"{
        "tenant_id": "tenant-family-01",
        "version": 4,
        "block_adult": true,
        "block_gambling": true,
        "block_violence": true,
        "block_social": false,
        "block_gaming": true,
        "safesearch_enabled": true,
        "youtube_restricted": true,
        "custom_blocked_domains": ["badgame.com", "distraction.net"],
        "custom_allowed_domains": ["school.edu"]
    }"#;

    let pol: WebFilterPolicyDto =
        serde_json::from_str(json).expect("Failed to deserialize WebFilterPolicyDto");
    assert_eq!(pol.tenant_id, "tenant-family-01");
    assert_eq!(pol.version, 4);
    assert!(pol.block_adult);
    assert!(pol.block_gambling);
    assert!(pol.block_gaming);
    assert!(!pol.block_social);
    assert!(!pol.silent_mode);
    assert_eq!(pol.dns_block_action, "nxdomain");
    assert_eq!(pol.custom_blocked_domains.len(), 2);
    assert_eq!(pol.custom_allowed_domains.len(), 1);
}

#[tokio::test]
async fn test_telemetry_sync_worker_standalone_mode_noop() {
    let storage = StorageEngine::in_memory().expect("Failed to open storage");
    let pipeline = Arc::new(TelemetryPipeline::new("dev-standalone-01", storage.clone()));
    let client = Arc::new(CloudSyncClient::new("http://127.0.0.1:9999"));
    let blocklist = Arc::new(BlocklistStore::new());

    let worker = TelemetrySyncWorker::new(
        pipeline.clone(),
        client,
        storage.clone(),
        Some(blocklist.clone()),
    );

    // Queue an event into outbox
    let now = chrono::Utc::now();
    pipeline
        .queue_dns_query("example.com", "A", "allow", "no_match", None, now)
        .expect("Failed to queue DNS event");

    assert_eq!(pipeline.pending_count().expect("Count"), 1);

    // Because device is NOT enrolled in cloud mode, running a shutdown loop must exit cleanly without panicking
    let (tx, rx) = tokio::sync::broadcast::channel(1);
    tx.send(()).expect("Failed to send shutdown signal");

    worker.run(rx).await;

    // Outbox event is safely preserved for offline operation
    assert_eq!(pipeline.pending_count().expect("Count"), 1);
}

#[tokio::test]
async fn test_web_filter_policy_update_dto_serialization() {
    let pol = WebFilterPolicyDto {
        tenant_id: "tenant-fam-99".to_string(),
        version: 5,
        block_adult: true,
        block_gambling: true,
        block_violence: false,
        block_social: false,
        block_gaming: true,
        safesearch_enabled: true,
        youtube_restricted: true,
        custom_blocked_domains: vec!["tiktok.com".to_string(), "roblox.com".to_string()],
        custom_allowed_domains: vec!["wikipedia.org".to_string()],
    };

    let serialized = serde_json::to_string(&pol).expect("Serialize policy DTO");
    assert!(serialized.contains("tenant-fam-99"));
    assert!(serialized.contains("tiktok.com"));

    let client = CloudSyncClient::new("http://127.0.0.1:54321");
    // Attempting to update to an unreachable server should return SyncError::Network
    let res = client.update_web_filter_policy("tenant-fam-99", &pol).await;
    assert!(res.is_err());
}

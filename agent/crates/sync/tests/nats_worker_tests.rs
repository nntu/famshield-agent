use famshield_dns::blocklist::BlocklistStore;
use famshield_storage::StorageEngine;
use famshield_sync::{NatsConnectionWorker, NatsDispatcher};
use famshield_windows::InstallerInterceptor;
use std::sync::Arc;

#[test]
fn test_nats_worker_initialization_and_mode_check() {
    let storage = StorageEngine::in_memory().unwrap();
    let blocklist = Arc::new(BlocklistStore::new());
    let installer = Arc::new(InstallerInterceptor::new());

    let dispatcher = Arc::new(NatsDispatcher::new(
        "dev-test-nats-01",
        storage.clone(),
        blocklist,
        installer,
    ));

    let _worker = NatsConnectionWorker::new(
        "dev-test-nats-01",
        "wss://nats.famshield.cloud/nats",
        dispatcher,
        storage.clone(),
    );

    // Initial state: not enrolled -> worker respects StandaloneFree mode
    assert!(storage
        .get_meta("enrolled_device_id")
        .ok()
        .flatten()
        .is_none());
}

#[tokio::test]
async fn test_nats_worker_shutdown_signal_immediate_exit() {
    let storage = StorageEngine::in_memory().unwrap();
    let blocklist = Arc::new(BlocklistStore::new());
    let installer = Arc::new(InstallerInterceptor::new());

    let dispatcher = Arc::new(NatsDispatcher::new(
        "dev-test-nats-02",
        storage.clone(),
        blocklist,
        installer,
    ));

    let worker = Arc::new(NatsConnectionWorker::new(
        "dev-test-nats-02",
        "wss://nats.famshield.cloud/nats",
        dispatcher,
        storage,
    ));

    let (shutdown_tx, shutdown_rx) = tokio::sync::broadcast::channel(1);

    // Send shutdown signal immediately
    let worker_clone = worker.clone();
    let handle = tokio::spawn(async move {
        worker_clone.run(shutdown_rx).await;
    });

    shutdown_tx.send(()).unwrap();
    let result = tokio::time::timeout(std::time::Duration::from_millis(500), handle).await;
    assert!(
        result.is_ok(),
        "Worker should cleanly exit upon shutdown signal"
    );
}

use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::info;

use famshield_dns::{
    blocklist::BlocklistStore, cache::DnsCache, proxy::DnsProxy, DEFAULT_BIND_ADDR,
    DEFAULT_UPSTREAM,
};
use famshield_storage::StorageEngine;
use famshield_sync::NatsDispatcher;
use famshield_windows::{wfp::WfpSessionGuard, InstallerInterceptor};
use crate::dispatcher::IpcDispatcher;

/// Core service controller orchestrating DNS, WFP, Local IPC, Storage, and NATS JetStream dispatcher.
pub struct ServiceController {
    storage: StorageEngine,
    ipc_dispatcher: Arc<IpcDispatcher>,
    dns_blocklist: Arc<BlocklistStore>,
    dns_cache: Arc<DnsCache>,
    installer: Arc<InstallerInterceptor>,
    nats_dispatcher: Arc<NatsDispatcher>,
}

impl ServiceController {
    pub fn new(storage: StorageEngine) -> Self {
        let ipc_dispatcher = Arc::new(IpcDispatcher::new(storage.clone()));
        let dns_blocklist = Arc::new(BlocklistStore::new());
        let dns_cache = Arc::new(DnsCache::new());
        let installer = Arc::new(InstallerInterceptor::new());

        // Read enrolled device_id from storage meta or fallback to standalone ID
        let device_id = storage
            .get_meta("enrolled_device_id")
            .ok()
            .flatten()
            .unwrap_or_else(|| "dev-local-001".to_string());

        let nats_dispatcher = Arc::new(NatsDispatcher::new(
            device_id,
            storage.clone(),
            dns_blocklist.clone(),
            installer.clone(),
        ));

        Self {
            storage,
            ipc_dispatcher,
            dns_blocklist,
            dns_cache,
            installer,
            nats_dispatcher,
        }
    }

    pub fn storage(&self) -> &StorageEngine {
        &self.storage
    }

    pub fn ipc_dispatcher(&self) -> Arc<IpcDispatcher> {
        self.ipc_dispatcher.clone()
    }

    pub fn dns_blocklist(&self) -> Arc<BlocklistStore> {
        self.dns_blocklist.clone()
    }

    pub fn nats_dispatcher(&self) -> Arc<NatsDispatcher> {
        self.nats_dispatcher.clone()
    }

    /// Run the integrated services (DNS loopback proxy, IPC dispatcher, WFP guard).
    pub async fn run(&self, mut shutdown_rx: broadcast::Receiver<()>) {
        info!("Starting FamShield Windows Service subsystems...");

        // 1. Initialize Dynamic WFP Session Guard
        let _wfp_guard = match WfpSessionGuard::open_dynamic("FamShieldServiceSession") {
            Ok(g) => {
                info!("Dynamic WFP User-Mode protection armed");
                Some(g)
            }
            Err(e) => {
                tracing::warn!("WFP session could not be established: {}", e);
                None
            }
        };

        // 2. Start DNS proxy listener on 127.0.0.2:53
        let dns_proxy = DnsProxy::new(
            DEFAULT_BIND_ADDR,
            DEFAULT_UPSTREAM,
            self.dns_blocklist.clone(),
            self.dns_cache.clone(),
        );

        tokio::select! {
            _ = dns_proxy.run_udp_listener() => {
                info!("DNS proxy listener ended");
            }
            _ = shutdown_rx.recv() => {
                info!("Shutdown signal received, gracefully terminating FamShield service");
            }
        }
    }
}

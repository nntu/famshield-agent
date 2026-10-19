use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::info;

use childprotect_dns::{
    blocklist::BlocklistStore, cache::DnsCache, proxy::DnsProxy, DEFAULT_BIND_ADDR,
    DEFAULT_UPSTREAM,
};
use childprotect_storage::StorageEngine;
use childprotect_windows::wfp::WfpSessionGuard;
use crate::dispatcher::IpcDispatcher;

/// Core service controller orchestrating DNS, WFP, Local IPC, and Storage.
pub struct ServiceController {
    storage: StorageEngine,
    ipc_dispatcher: Arc<IpcDispatcher>,
    dns_blocklist: Arc<BlocklistStore>,
    dns_cache: Arc<DnsCache>,
}

impl ServiceController {
    pub fn new(storage: StorageEngine) -> Self {
        let ipc_dispatcher = Arc::new(IpcDispatcher::new(storage.clone()));
        let dns_blocklist = Arc::new(BlocklistStore::new());
        let dns_cache = Arc::new(DnsCache::new());

        Self {
            storage,
            ipc_dispatcher,
            dns_blocklist,
            dns_cache,
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

    /// Run the integrated services (DNS loopback proxy, IPC dispatcher, WFP guard).
    pub async fn run(&self, mut shutdown_rx: broadcast::Receiver<()>) {
        info!("Starting ChildProtect Windows Service subsystems...");

        // 1. Initialize Dynamic WFP Session Guard
        let _wfp_guard = match WfpSessionGuard::open_dynamic("ChildProtectServiceSession") {
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
                info!("Shutdown signal received, gracefully terminating ChildProtect service");
            }
        }
    }
}

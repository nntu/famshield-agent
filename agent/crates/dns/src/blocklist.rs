use arc_swap::ArcSwap;
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Clone)]
pub struct BlocklistStore {
    // Exact domain or parent suffix set (e.g., "tiktok.com")
    domains: Arc<ArcSwap<HashSet<String>>>,
    monitor_mode: Arc<AtomicBool>,
}

impl Default for BlocklistStore {
    fn default() -> Self {
        Self::new()
    }
}

impl BlocklistStore {
    pub fn new() -> Self {
        Self {
            domains: Arc::new(ArcSwap::from_pointee(HashSet::new())),
            monitor_mode: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Enable or disable Monitor-only Mode (Audit mode: log only, zero blocking).
    pub fn set_monitor_mode(&self, enabled: bool) {
        self.monitor_mode.store(enabled, Ordering::SeqCst);
    }

    /// Check if Monitor Mode is currently active.
    pub fn is_monitor_mode(&self) -> bool {
        self.monitor_mode.load(Ordering::SeqCst)
    }

    /// Check if target domain should actually be blocked (blocked AND not in monitor mode).
    pub fn should_block(&self, target_domain: &str) -> bool {
        !self.is_monitor_mode() && self.is_blocked(target_domain)
    }

    /// Load or atomically replace the blocked domains list without locking reader threads.
    pub fn replace_blocklist<I, S>(&self, iter: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut set = HashSet::new();
        for item in iter {
            let s = item
                .as_ref()
                .trim()
                .trim_end_matches('.')
                .to_ascii_lowercase();
            if !s.is_empty() {
                set.insert(s);
            }
        }
        self.domains.store(Arc::new(set));
    }

    /// Check if target domain matches blocklist via exact match or parent subdomain suffix.
    pub fn is_blocked(&self, target_domain: &str) -> bool {
        let current = self.domains.load();
        let norm = target_domain
            .trim()
            .trim_end_matches('.')
            .to_ascii_lowercase();

        if current.contains(&norm) {
            return true;
        }

        // Subdomain traversal: e.g. "v16.tiktok.com" -> check "tiktok.com", "com"
        let mut s = norm.as_str();
        while let Some(idx) = s.find('.') {
            s = &s[idx + 1..];
            if current.contains(s) {
                return true;
            }
        }

        false
    }
}

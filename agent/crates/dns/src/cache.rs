use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
pub struct CachedResponse {
    pub wire: Vec<u8>,
    pub created_at: Instant,
    pub original_ttl: Duration,
}

impl CachedResponse {
    pub fn is_fresh(&self, now: Instant) -> bool {
        now.duration_since(self.created_at) < self.original_ttl
    }

    /// Serve-Stale allows serving expired records during upstream outage (RFC 8767, up to 7 days).
    pub fn is_stale_usable(&self, now: Instant) -> bool {
        let age = now.duration_since(self.created_at);
        age < Duration::from_secs(7 * 24 * 3600)
    }

    /// True if nearing TTL expiration (<10% remaining), eligible for background prefetch.
    pub fn needs_prefetch(&self, now: Instant) -> bool {
        let age = now.duration_since(self.created_at);
        if age >= self.original_ttl {
            return false;
        }
        let remaining = self.original_ttl - age;
        remaining < (self.original_ttl / 10)
    }
}

#[derive(Clone, Default)]
pub struct DnsCache {
    entries: Arc<RwLock<HashMap<String, CachedResponse>>>,
}

impl DnsCache {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn get(&self, key: &str, now: Instant) -> Option<(Vec<u8>, bool)> {
        let map = self.entries.read().ok()?;
        let entry = map.get(key)?;
        if entry.is_fresh(now) {
            Some((entry.wire.clone(), false)) // Fresh
        } else if entry.is_stale_usable(now) {
            Some((entry.wire.clone(), true)) // Stale (RFC 8767)
        } else {
            None
        }
    }

    pub fn insert(&self, key: String, wire: Vec<u8>, ttl_secs: u32, now: Instant) {
        let mut map = match self.entries.write() {
            Ok(m) => m,
            Err(_) => return,
        };
        // Clamp TTL between 5 seconds and 1 day
        let clamped_ttl = ttl_secs.clamp(5, 86400);
        map.insert(
            key,
            CachedResponse {
                wire,
                created_at: now,
                original_ttl: Duration::from_secs(clamped_ttl as u64),
            },
        );
    }
}

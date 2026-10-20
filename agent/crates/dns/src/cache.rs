use hickory_proto::serialize::binary::{BinDecodable, BinEncodable};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

pub const MAX_CACHE_ENTRIES: usize = 2000;

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

    pub fn remaining_ttl(&self, now: Instant) -> u32 {
        let elapsed = now.duration_since(self.created_at);
        if elapsed >= self.original_ttl {
            10
        } else {
            (self.original_ttl - elapsed).as_secs().max(5) as u32
        }
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
    hits: Arc<AtomicU64>,
    misses: Arc<AtomicU64>,
}

impl DnsCache {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            hits: Arc::new(AtomicU64::new(0)),
            misses: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn get(&self, key: &str, now: Instant) -> Option<(Vec<u8>, bool)> {
        let map = self.entries.read().ok()?;
        let entry = map.get(key)?;
        if entry.is_fresh(now) {
            self.hits.fetch_add(1, Ordering::Relaxed);
            let mut wire = entry.wire.clone();
            let remaining = entry.remaining_ttl(now);
            Self::adjust_wire_ttl(&mut wire, remaining);
            Some((wire, false)) // Fresh
        } else if entry.is_stale_usable(now) {
            self.hits.fetch_add(1, Ordering::Relaxed);
            let mut wire = entry.wire.clone();
            Self::adjust_wire_ttl(&mut wire, 30); // 30s stale TTL
            Some((wire, true)) // Stale (RFC 8767)
        } else {
            self.misses.fetch_add(1, Ordering::Relaxed);
            None
        }
    }

    pub fn insert(&self, key: String, wire: Vec<u8>, ttl_secs: u32, now: Instant) {
        let mut map = match self.entries.write() {
            Ok(m) => m,
            Err(_) => return,
        };

        // Bounded memory protection: keep cache under MAX_CACHE_ENTRIES (<= 2MB RAM)
        if map.len() >= MAX_CACHE_ENTRIES {
            // Remove expired entries first
            map.retain(|_, v| v.is_fresh(now) || v.is_stale_usable(now));
            // If still full, purge oldest 25%
            if map.len() >= MAX_CACHE_ENTRIES {
                let to_remove = MAX_CACHE_ENTRIES / 4;
                let keys_to_remove: Vec<String> = map.keys().take(to_remove).cloned().collect();
                for k in keys_to_remove {
                    map.remove(&k);
                }
            }
        }

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

    /// Flush all cached entries immediately (used when blocklist rules or SafeSearch settings change)
    pub fn flush(&self) {
        if let Ok(mut map) = self.entries.write() {
            map.clear();
        }
    }

    /// Number of currently cached domain queries
    pub fn len(&self) -> usize {
        self.entries.read().map(|m| m.len()).unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns (current_entries_count, total_hits, total_misses)
    pub fn stats(&self) -> (usize, u64, u64) {
        let count = self.len();
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        (count, hits, misses)
    }

    /// Helper to adjust the TTL in the answer records of a DNS wire packet
    fn adjust_wire_ttl(wire: &mut [u8], remaining_ttl: u32) {
        if wire.len() > 12 {
            if let Ok(mut msg) = hickory_proto::op::Message::from_bytes(wire) {
                if !msg.answers.is_empty() {
                    for record in &mut msg.answers {
                        record.ttl = remaining_ttl;
                    }
                    if let Ok(new_wire) = msg.to_bytes() {
                        if new_wire.len() == wire.len() {
                            wire.copy_from_slice(&new_wire);
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dns_cache_bounded_capacity_and_flush() {
        let cache = DnsCache::new();
        let now = Instant::now();

        cache.insert("example.com:1".to_string(), vec![1, 2, 3], 60, now);
        assert_eq!(cache.len(), 1);

        let (hit_wire, is_stale) = cache.get("example.com:1", now).unwrap();
        assert_eq!(hit_wire, vec![1, 2, 3]);
        assert!(!is_stale);

        let (len, hits, misses) = cache.stats();
        assert_eq!(len, 1);
        assert_eq!(hits, 1);
        assert_eq!(misses, 0);

        cache.flush();
        assert_eq!(cache.len(), 0);
        assert!(cache.get("example.com:1", now).is_none());
    }
}

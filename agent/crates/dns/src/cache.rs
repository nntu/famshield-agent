use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

pub const MAX_CACHE_ENTRIES: usize = 2000;
const TYPE_OPT: u16 = 41;
const TC_FLAG: u8 = 0x02;

#[derive(Clone, Debug)]
pub struct CachedResponse {
    pub wire: Vec<u8>,
    pub ttl_offsets: Vec<usize>,
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
        self.get_with_prefetch(key, now)
            .map(|(wire, is_stale, _)| (wire, is_stale))
    }

    /// Returns `(wire_bytes, is_stale, needs_prefetch)` for a cached DNS response.
    pub fn get_with_prefetch(&self, key: &str, now: Instant) -> Option<(Vec<u8>, bool, bool)> {
        let map = self.entries.read().ok()?;
        let entry = map.get(key)?;
        if entry.is_fresh(now) {
            self.hits.fetch_add(1, Ordering::Relaxed);
            let mut wire = entry.wire.clone();
            let remaining = entry.remaining_ttl(now);
            patch_ttls(&mut wire, &entry.ttl_offsets, remaining);
            let prefetch = entry.needs_prefetch(now);
            Some((wire, false, prefetch)) // Fresh
        } else if entry.is_stale_usable(now) && !is_failure_wire(&entry.wire) {
            self.hits.fetch_add(1, Ordering::Relaxed);
            let mut wire = entry.wire.clone();
            patch_ttls(&mut wire, &entry.ttl_offsets, 30); // 30s stale TTL
            Some((wire, true, false)) // Stale (RFC 8767)
        } else {
            self.misses.fetch_add(1, Ordering::Relaxed);
            None
        }
    }

    /// Extracts the minimum answer TTL (in seconds, clamped to `[5, 86400]`) from a DNS response wire packet
    /// via zero-parse wire scanning (from `numa`). Falls back to `300` seconds if no answer records exist.
    pub fn extract_min_ttl(wire: &[u8]) -> u32 {
        if let Some((offsets, answer_count)) = scan_ttl_offsets(wire) {
            if let Some(min_ttl) = offsets
                .iter()
                .take(answer_count)
                .filter_map(|&off| {
                    wire.get(off..off + 4)
                        .map(|b| u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
                })
                .min()
            {
                return min_ttl.clamp(5, 86400);
            }
        }
        300
    }

    pub fn insert(&self, key: String, wire: Vec<u8>, ttl_secs: u32, now: Instant) {
        // Never cache truncated (TC=1) responses (RFC 1035 / numa)
        if wire.len() >= 12 && wire[2] & TC_FLAG != 0 {
            return;
        }

        let ttl_offsets = scan_ttl_offsets(&wire)
            .map(|(offsets, _)| offsets)
            .unwrap_or_default();

        let mut map = match self.entries.write() {
            Ok(m) => m,
            Err(_) => return,
        };

        // Bounded memory protection: keep cache under MAX_CACHE_ENTRIES (<= 2MB RAM)
        if map.len() >= MAX_CACHE_ENTRIES {
            // Remove expired entries first
            map.retain(|_, v| v.is_fresh(now) || v.is_stale_usable(now));
            // If still full, purge oldest 25% by creation timestamp
            if map.len() >= MAX_CACHE_ENTRIES {
                let to_remove = MAX_CACHE_ENTRIES / 4;
                let mut entries_by_age: Vec<(String, Instant)> =
                    map.iter().map(|(k, v)| (k.clone(), v.created_at)).collect();
                entries_by_age.sort_by_key(|(_, created_at)| *created_at);
                for (k, _) in entries_by_age.into_iter().take(to_remove) {
                    map.remove(&k);
                }
            }
        }

        let clamped_ttl = ttl_secs.clamp(5, 86400);
        map.insert(
            key,
            CachedResponse {
                wire,
                ttl_offsets,
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
}

#[inline]
fn is_failure_wire(wire: &[u8]) -> bool {
    wire.len() >= 12 && !matches!(wire[3] & 0x0F, 0 | 3)
}

#[inline]
fn patch_ttls(wire: &mut [u8], offsets: &[usize], new_ttl: u32) {
    let bytes = new_ttl.to_be_bytes();
    for &off in offsets {
        if off + 4 <= wire.len() {
            wire[off..off + 4].copy_from_slice(&bytes);
        }
    }
}

/// Scans raw DNS wire bytes for all non-OPT resource record TTL offsets without full packet parsing.
/// Returns `(ttl_offsets, answer_count)`.
fn scan_ttl_offsets(wire: &[u8]) -> Option<(Vec<usize>, usize)> {
    if wire.len() < 12 {
        return None;
    }

    let mut pos = 12usize;
    let qdcount = u16::from_be_bytes([wire[4], wire[5]]) as usize;
    for _ in 0..qdcount {
        skip_wire_name(wire, &mut pos)?;
        if pos + 4 > wire.len() {
            return None;
        }
        pos += 4;
    }

    let mut ttl_offsets = Vec::new();
    let mut answer_count = 0usize;

    for (section, at) in [6usize, 8, 10].into_iter().enumerate() {
        let rrcount = u16::from_be_bytes([wire[at], wire[at + 1]]) as usize;
        for _ in 0..rrcount {
            skip_wire_name(wire, &mut pos)?;
            if pos + 10 > wire.len() {
                return None;
            }
            let rtype = u16::from_be_bytes([wire[pos], wire[pos + 1]]);
            let ttl_offset = pos + 4;
            let rdlength = u16::from_be_bytes([wire[pos + 8], wire[pos + 9]]) as usize;
            pos += 10 + rdlength;
            if pos > wire.len() {
                return None;
            }
            if rtype != TYPE_OPT {
                ttl_offsets.push(ttl_offset);
                if section == 0 {
                    answer_count += 1;
                }
            }
        }
    }

    Some((ttl_offsets, answer_count))
}

fn skip_wire_name(wire: &[u8], pos: &mut usize) -> Option<()> {
    let mut labels = 0usize;
    loop {
        if *pos >= wire.len() || labels > 128 {
            return None;
        }
        let len = wire[*pos] as usize;
        if len & 0xC0 == 0xC0 {
            if *pos + 2 > wire.len() {
                return None;
            }
            *pos += 2;
            return Some(());
        }
        if len == 0 {
            *pos += 1;
            return Some(());
        }
        if len > 63 {
            return None;
        }
        *pos += 1 + len;
        labels += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hickory_proto::serialize::binary::{BinDecodable, BinEncodable};

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

    #[test]
    fn test_dns_cache_prefetch_and_extract_min_ttl() {
        use hickory_proto::op::{Message, MessageType, OpCode, Query};
        use hickory_proto::rr::{rdata, Name, RData, Record, RecordType};
        use std::str::FromStr;

        let mut msg = Message::new(0x1111, MessageType::Response, OpCode::Query);
        let name = Name::from_str("ttl-test.example.").unwrap();
        msg.add_query(Query::query(name.clone(), RecordType::A));
        msg.add_answer(Record::from_rdata(
            name.clone(),
            120,
            RData::A(rdata::A(std::net::Ipv4Addr::new(1, 2, 3, 4))),
        ));
        msg.add_answer(Record::from_rdata(
            name,
            45,
            RData::A(rdata::A(std::net::Ipv4Addr::new(5, 6, 7, 8))),
        ));
        let wire = msg.to_bytes().unwrap();
        assert_eq!(DnsCache::extract_min_ttl(&wire), 45);
        assert_eq!(DnsCache::extract_min_ttl(&[1, 2, 3]), 300);

        let cache = DnsCache::new();
        let start = Instant::now();
        cache.insert("ttl-test.example:1".to_string(), wire, 100, start);

        let (_, is_stale, needs_prefetch) = cache
            .get_with_prefetch("ttl-test.example:1", start)
            .unwrap();
        assert!(!is_stale);
        assert!(!needs_prefetch);

        // At 95s elapsed (<10% of 100s TTL remaining), prefetch should trigger
        let near_expiry = start + Duration::from_secs(95);
        let (patched_wire, is_stale_near, needs_prefetch_near) = cache
            .get_with_prefetch("ttl-test.example:1", near_expiry)
            .unwrap();
        assert!(!is_stale_near);
        assert!(needs_prefetch_near);

        // Verify TTL was patched in-place to 5s (minimum remaining clamp)
        let parsed = Message::from_bytes(&patched_wire).unwrap();
        assert_eq!(parsed.answers[0].ttl, 5);
        assert_eq!(parsed.answers[1].ttl, 5);

        // Truncated (TC=1) responses must never be cached
        let mut tc_wire = patched_wire;
        tc_wire[2] |= 0x02;
        cache.insert("tc-skip.example:1".to_string(), tc_wire, 60, start);
        assert!(cache.get("tc-skip.example:1", start).is_none());
    }
}

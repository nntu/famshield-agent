use std::sync::Arc;
use std::time::Instant;
use childprotect_dns::{
    blocklist::BlocklistStore,
    buffer::BytePacketBuffer,
    cache::DnsCache,
    proxy::DnsProxy,
};

#[test]
fn test_byte_packet_buffer_qname_read_write() {
    let mut buf = BytePacketBuffer::new();
    let domain = "sub.example.com";
    buf.write_qname(domain).unwrap();

    let mut read_buf = BytePacketBuffer::from_bytes(buf.filled());
    let mut out = String::new();
    read_buf.read_qname(&mut out).unwrap();

    assert_eq!(out, domain);
}

#[test]
fn test_blocklist_exact_and_subdomain_matching() {
    let blocklist = BlocklistStore::new();
    blocklist.replace_blocklist(vec!["tiktok.com", "gambling.org"]);

    assert!(blocklist.is_blocked("tiktok.com"));
    assert!(blocklist.is_blocked("v16.tiktok.com"));
    assert!(blocklist.is_blocked("sub.v16.tiktok.com"));
    assert!(blocklist.is_blocked("gambling.org"));

    assert!(!blocklist.is_blocked("example.com"));
    assert!(!blocklist.is_blocked("not-tiktok.com"));
}

#[test]
fn test_serve_stale_dns_cache() {
    let cache = DnsCache::new();
    let key = "example.com:1".to_string();
    let wire = vec![1, 2, 3, 4];
    let now = Instant::now();

    cache.insert(key.clone(), wire.clone(), 10, now);

    let (found, is_stale) = cache.get(&key, now).expect("Should find cached item");
    assert_eq!(found, wire);
    assert!(!is_stale);

    // Simulate elapsed time past TTL (e.g. 15s later)
    let later = now + std::time::Duration::from_secs(15);
    let (found_stale, is_stale_later) = cache.get(&key, later).expect("Should find stale item (RFC 8767)");
    assert_eq!(found_stale, wire);
    assert!(is_stale_later);
}

#[test]
fn test_build_blocked_response_format() {
    let mut query = BytePacketBuffer::new();
    query.write_u16(0x5678).unwrap(); // ID
    query.write_u16(0x0100).unwrap(); // Standard query flags
    query.write_u16(1).unwrap();      // 1 question
    query.write_u16(0).unwrap();
    query.write_u16(0).unwrap();
    query.write_u16(0).unwrap();
    query.write_qname("ads.blocked.net").unwrap();
    query.write_u16(1).unwrap(); // A
    query.write_u16(1).unwrap(); // IN

    let blocked_wire = DnsProxy::build_blocked_response(query.filled()).unwrap();

    let mut resp = BytePacketBuffer::from_bytes(&blocked_wire);
    let id = resp.read_u16().unwrap();
    let flags = resp.read_u16().unwrap();

    assert_eq!(id, 0x5678);
    // Assert response flag QR=1, RCODE=3 (NXDOMAIN)
    assert_eq!(flags & 0x8000, 0x8000);
    assert_eq!(flags & 0x000F, 3);
}

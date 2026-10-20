use famshield_dns::{
    blocklist::BlocklistStore, buffer::BytePacketBuffer, cache::DnsCache, proxy::DnsProxy,
};
use hickory_proto::serialize::binary::BinDecodable;
use std::time::Instant;

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
    let (found_stale, is_stale_later) = cache
        .get(&key, later)
        .expect("Should find stale item (RFC 8767)");
    assert_eq!(found_stale, wire);
    assert!(is_stale_later);
}

#[test]
fn test_build_blocked_response_format() {
    let mut query = BytePacketBuffer::new();
    query.write_u16(0x5678).unwrap(); // ID
    query.write_u16(0x0100).unwrap(); // Standard query flags
    query.write_u16(1).unwrap(); // 1 question
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

#[test]
fn test_build_servfail_response_format() {
    let mut query = BytePacketBuffer::new();
    query.write_u16(0x7890).unwrap(); // ID
    query.write_u16(0x0100).unwrap(); // Standard query flags
    query.write_u16(1).unwrap(); // 1 question
    query.write_u16(0).unwrap();
    query.write_u16(0).unwrap();
    query.write_u16(0).unwrap();
    query.write_qname("fail.test.org").unwrap();
    query.write_u16(1).unwrap(); // A
    query.write_u16(1).unwrap(); // IN

    let servfail_wire = DnsProxy::build_servfail_response(query.filled()).unwrap();

    let mut resp = BytePacketBuffer::from_bytes(&servfail_wire);
    let id = resp.read_u16().unwrap();
    let flags = resp.read_u16().unwrap();

    assert_eq!(id, 0x7890);
    // Assert response flag QR=1, RCODE=2 (SERVFAIL)
    assert_eq!(flags & 0x8000, 0x8000);
    assert_eq!(flags & 0x000F, 2);
}

#[tokio::test]
async fn test_dns_proxy_forwarding() {
    let blocklist = std::sync::Arc::new(famshield_dns::BlocklistStore::new());
    let cache = std::sync::Arc::new(famshield_dns::DnsCache::new());
    let proxy = DnsProxy::new("127.0.0.1:53535", "1.1.1.1:53", blocklist, cache);

    let proxy_handle = tokio::spawn(async move {
        let _ = proxy.run_udp_listener().await;
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let client = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
    // Build a query for google.com
    let mut query = BytePacketBuffer::new();
    query.write_u16(0x1111).unwrap();
    query.write_u16(0x0100).unwrap();
    query.write_u16(1).unwrap();
    query.write_u16(0).unwrap();
    query.write_u16(0).unwrap();
    query.write_u16(0).unwrap();
    query.write_qname("google.com").unwrap();
    query.write_u16(1).unwrap();
    query.write_u16(1).unwrap();

    client
        .send_to(query.filled(), "127.0.0.1:53535")
        .await
        .unwrap();

    let mut buf = [0u8; 512];
    let res = tokio::time::timeout(
        std::time::Duration::from_secs(3),
        client.recv_from(&mut buf),
    )
    .await;
    println!("Result: {:?}", res);
    assert!(res.is_ok(), "Client timed out waiting for DNS response!");
    proxy_handle.abort();
}

#[tokio::test]
async fn test_dns_proxy_blocked_domain() {
    let blocklist = std::sync::Arc::new(famshield_dns::BlocklistStore::new());
    blocklist.replace_blocklist(vec!["badsite.com".to_string()]);
    let cache = std::sync::Arc::new(famshield_dns::DnsCache::new());
    let (log_tx, mut log_rx) = tokio::sync::mpsc::unbounded_channel();
    let proxy =
        DnsProxy::new("127.0.0.1:53536", "1.1.1.1:53", blocklist, cache).with_log_sender(log_tx);

    let proxy_handle = tokio::spawn(async move {
        let _ = proxy.run_udp_listener().await;
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let client = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
    // Build a query for badsite.com
    let mut query = BytePacketBuffer::new();
    query.write_u16(0x2222).unwrap();
    query.write_u16(0x0100).unwrap();
    query.write_u16(1).unwrap();
    query.write_u16(0).unwrap();
    query.write_u16(0).unwrap();
    query.write_u16(0).unwrap();
    query.write_qname("badsite.com").unwrap();
    query.write_u16(1).unwrap();
    query.write_u16(1).unwrap();

    client
        .send_to(query.filled(), "127.0.0.1:53536")
        .await
        .unwrap();

    let mut buf = [0u8; 512];
    let res = tokio::time::timeout(
        std::time::Duration::from_secs(3),
        client.recv_from(&mut buf),
    )
    .await;
    assert!(
        res.is_ok(),
        "Client timed out waiting for blocked response!"
    );
    let (len, _) = res.unwrap().unwrap();
    let resp = &buf[..len];
    let resp_msg = hickory_proto::op::Message::from_bytes(resp).unwrap();
    assert_eq!(
        resp_msg.metadata.response_code,
        hickory_proto::op::ResponseCode::NXDomain
    );
    let log = tokio::time::timeout(std::time::Duration::from_secs(1), log_rx.recv())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(log.domain, "badsite.com");
    assert_eq!(log.action, "BLOCK");
    assert_eq!(log.reason, "local_domain_rule");
    proxy_handle.abort();
}

#[tokio::test]
async fn test_dns_proxy_monitor_mode_logs_without_blocking() {
    let upstream = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let upstream_addr = upstream.local_addr().unwrap();
    let upstream_handle = tokio::spawn(async move {
        let mut buf = [0_u8; 512];
        let (len, peer) = upstream.recv_from(&mut buf).await.unwrap();
        let mut response = buf[..len].to_vec();
        response[2] |= 0x80; // QR=response
        upstream.send_to(&response, peer).await.unwrap();
    });

    let blocklist = std::sync::Arc::new(famshield_dns::BlocklistStore::new());
    blocklist.replace_blocklist(vec!["monitored.example".to_string()]);
    blocklist.set_monitor_mode(true);
    let cache = std::sync::Arc::new(famshield_dns::DnsCache::new());
    let (log_tx, mut log_rx) = tokio::sync::mpsc::unbounded_channel();
    let proxy = DnsProxy::new(
        "127.0.0.1:53537",
        upstream_addr.to_string(),
        blocklist,
        cache,
    )
    .with_log_sender(log_tx);
    let proxy_handle = tokio::spawn(async move {
        let _ = proxy.run_udp_listener().await;
    });
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let client = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let mut query = BytePacketBuffer::new();
    query.write_u16(0x3333).unwrap();
    query.write_u16(0x0100).unwrap();
    query.write_u16(1).unwrap();
    query.write_u16(0).unwrap();
    query.write_u16(0).unwrap();
    query.write_u16(0).unwrap();
    query.write_qname("monitored.example").unwrap();
    query.write_u16(1).unwrap();
    query.write_u16(1).unwrap();
    client
        .send_to(query.filled(), "127.0.0.1:53537")
        .await
        .unwrap();

    let mut response = [0_u8; 512];
    tokio::time::timeout(
        std::time::Duration::from_secs(1),
        client.recv_from(&mut response),
    )
    .await
    .unwrap()
    .unwrap();
    let log = tokio::time::timeout(std::time::Duration::from_secs(1), log_rx.recv())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(log.domain, "monitored.example");
    assert_eq!(log.action, "MONITOR");
    assert_eq!(log.reason, "local_domain_rule");

    proxy_handle.abort();
    upstream_handle.await.unwrap();
}

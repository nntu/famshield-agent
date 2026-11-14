use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::net::UdpSocket;
use tokio::sync::{Mutex, Notify};
use tracing::{debug, info, warn};

use crate::balancer::NumaDnsBalancer;
use crate::blocklist::BlocklistStore;
use crate::buffer::BytePacketBuffer;
use crate::cache::DnsCache;
use crate::error::DnsError;
use crate::program_lookup::PortProgramLookup;
use hickory_proto::op::{Message, MessageType, ResponseCode};
use hickory_proto::serialize::binary::{BinDecodable, BinEncodable};

pub const DEFAULT_BIND_ADDR: &str = famshield_core::DEFAULT_DNS_BIND_ADDR;
pub const DEFAULT_UPSTREAM: &str = famshield_core::DEFAULT_DNS_UPSTREAM_ADDR;
const MAX_UPSTREAM_SOCKET_POOL: usize = 32;

#[derive(Clone, Debug)]
pub struct DnsLogEntry {
    pub domain: String,
    pub qtype: String,
    pub action: String,
    pub reason: String,
    pub rule_id: Option<String>,
    pub process_name: Option<String>,
}

pub fn qtype_to_string(qtype: u16) -> &'static str {
    match qtype {
        1 => "A",
        28 => "AAAA",
        5 => "CNAME",
        12 => "PTR",
        15 => "MX",
        16 => "TXT",
        6 => "SOA",
        2 => "NS",
        33 => "SRV",
        65 => "HTTPS",
        _ => "OTHER",
    }
}

#[derive(Clone, Default)]
struct UpstreamSocketPool {
    sockets: Arc<Mutex<Vec<UdpSocket>>>,
}

impl UpstreamSocketPool {
    async fn acquire(&self) -> Result<UdpSocket, DnsError> {
        if let Some(sock) = self.sockets.lock().await.pop() {
            // Drain any stale datagrams left over from a previously timed-out query
            let mut discard = [0u8; 512];
            while sock.try_recv_from(&mut discard).is_ok() {}
            return Ok(sock);
        }
        Ok(UdpSocket::bind("0.0.0.0:0").await?)
    }

    async fn release(&self, sock: UdpSocket) {
        let mut guard = self.sockets.lock().await;
        if guard.len() < MAX_UPSTREAM_SOCKET_POOL {
            guard.push(sock);
        }
    }
}

type InFlightMap = Arc<Mutex<HashMap<String, Arc<Notify>>>>;

/// Lock-free DNS path counters modeled after `numa::stats::ServerStats`.
#[derive(Default)]
pub struct DnsStats {
    pub queries_total: AtomicU64,
    pub queries_blocked: AtomicU64,
    pub queries_rewritten: AtomicU64,
    pub queries_cached: AtomicU64,
    pub queries_stale: AtomicU64,
    pub queries_coalesced: AtomicU64,
    pub queries_upstream: AtomicU64,
    pub queries_error: AtomicU64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DnsStatsSnapshot {
    pub queries_total: u64,
    pub queries_blocked: u64,
    pub queries_rewritten: u64,
    pub queries_cached: u64,
    pub queries_stale: u64,
    pub queries_coalesced: u64,
    pub queries_upstream: u64,
    pub queries_error: u64,
}

impl DnsStats {
    pub fn snapshot(&self) -> DnsStatsSnapshot {
        DnsStatsSnapshot {
            queries_total: self.queries_total.load(Ordering::Relaxed),
            queries_blocked: self.queries_blocked.load(Ordering::Relaxed),
            queries_rewritten: self.queries_rewritten.load(Ordering::Relaxed),
            queries_cached: self.queries_cached.load(Ordering::Relaxed),
            queries_stale: self.queries_stale.load(Ordering::Relaxed),
            queries_coalesced: self.queries_coalesced.load(Ordering::Relaxed),
            queries_upstream: self.queries_upstream.load(Ordering::Relaxed),
            queries_error: self.queries_error.load(Ordering::Relaxed),
        }
    }
}

pub struct DnsProxy {
    bind_addr: String,
    balancer: Arc<NumaDnsBalancer>,
    blocklist: Arc<BlocklistStore>,
    cache: Arc<DnsCache>,
    log_tx: Option<tokio::sync::mpsc::UnboundedSender<DnsLogEntry>>,
    socket_pool: UpstreamSocketPool,
    in_flight: InFlightMap,
    program_lookup: PortProgramLookup,
    stats: Arc<DnsStats>,
}

impl DnsProxy {
    pub fn new(
        bind_addr: impl Into<String>,
        upstream_addr: impl Into<String>,
        blocklist: Arc<BlocklistStore>,
        cache: Arc<DnsCache>,
    ) -> Self {
        let addr = upstream_addr.into();
        Self {
            bind_addr: bind_addr.into(),
            balancer: Arc::new(NumaDnsBalancer::new(&[addr])),
            blocklist,
            cache,
            log_tx: None,
            socket_pool: UpstreamSocketPool::default(),
            in_flight: Arc::new(Mutex::new(HashMap::new())),
            program_lookup: PortProgramLookup::new(),
            stats: Arc::new(DnsStats::default()),
        }
    }

    pub fn stats(&self) -> DnsStatsSnapshot {
        self.stats.snapshot()
    }

    pub fn with_upstreams(self, upstreams: Vec<String>) -> Self {
        self.balancer.set_upstreams(&upstreams);
        self
    }

    pub fn with_balancer(mut self, balancer: Arc<NumaDnsBalancer>) -> Self {
        self.balancer = balancer;
        self
    }

    pub fn set_upstreams(&self, upstreams: Vec<String>) {
        self.balancer.set_upstreams(&upstreams);
    }

    pub fn get_upstreams(&self) -> Vec<String> {
        self.balancer
            .get_diagnostics()
            .into_iter()
            .map(|d| d.target)
            .collect()
    }

    pub fn balancer(&self) -> Arc<NumaDnsBalancer> {
        self.balancer.clone()
    }

    pub fn with_log_sender(mut self, tx: tokio::sync::mpsc::UnboundedSender<DnsLogEntry>) -> Self {
        self.log_tx = Some(tx);
        self
    }

    /// Shared helper to build an empty-answer DNS reply from an already-parsed query `Message`,
    /// falling back to `BytePacketBuffer` when `parsed` is `None`.
    fn build_empty_reply(
        query_packet: &[u8],
        parsed: Option<&Message>,
        rcode: ResponseCode,
        authoritative: bool,
        fallback_flags: u16,
    ) -> Result<Vec<u8>, DnsError> {
        if let Some(query_msg) = parsed {
            let mut response = Message::new(
                query_msg.metadata.id,
                MessageType::Response,
                query_msg.metadata.op_code,
            );
            response.metadata.recursion_desired = query_msg.metadata.recursion_desired;
            response.metadata.authoritative = authoritative;
            response.metadata.recursion_available = true;
            response.metadata.response_code = rcode;
            response.add_queries(query_msg.queries.clone());

            if let Ok(wire) = response.to_bytes() {
                return Ok(wire);
            }
        }

        if query_packet.len() < 12 {
            return Err(DnsError::EndOfBuffer);
        }

        let mut out = BytePacketBuffer::new();
        out.write_u8(query_packet[0])?;
        out.write_u8(query_packet[1])?;
        out.write_u16(fallback_flags)?;
        out.write_u16(1)?;
        out.write_u16(0)?;
        out.write_u16(0)?;
        out.write_u16(0)?;

        let mut in_buf = BytePacketBuffer::from_bytes(query_packet);
        in_buf.seek(12)?;
        let mut qname = String::new();
        in_buf.read_qname(&mut qname)?;
        let qtype = in_buf.read_u16()?;
        let qclass = in_buf.read_u16()?;

        out.write_qname(&qname)?;
        out.write_u16(qtype)?;
        out.write_u16(qclass)?;

        Ok(out.filled().to_vec())
    }

    /// Builds a synthetic NXDOMAIN or 0.0.0.0 blocked DNS response wire using hickory-proto.
    pub fn build_blocked_response(query_packet: &[u8]) -> Result<Vec<u8>, DnsError> {
        let parsed = Message::from_bytes(query_packet).ok();
        Self::build_empty_reply(
            query_packet,
            parsed.as_ref(),
            ResponseCode::NXDomain,
            true,
            0x8183,
        )
    }

    /// Constructs an authoritative A-record response for SafeSearch VIP rewrites.
    pub fn build_a_record_response(
        query_packet: &[u8],
        ip: std::net::Ipv4Addr,
    ) -> Result<Vec<u8>, DnsError> {
        let parsed = Message::from_bytes(query_packet).ok();
        Self::build_a_record_response_with_msg(query_packet, parsed.as_ref(), ip)
    }

    fn build_a_record_response_with_msg(
        query_packet: &[u8],
        parsed: Option<&Message>,
        ip: std::net::Ipv4Addr,
    ) -> Result<Vec<u8>, DnsError> {
        // 1. Attempt RFC-compliant response generation via pre-parsed hickory-proto Message
        if let Some(query_msg) = parsed {
            if let Some(query) = query_msg.queries.first() {
                let mut response = Message::new(
                    query_msg.metadata.id,
                    MessageType::Response,
                    query_msg.metadata.op_code,
                );
                response.metadata.recursion_desired = query_msg.metadata.recursion_desired;
                response.metadata.authoritative = true;
                response.metadata.recursion_available = true;
                response.metadata.response_code = ResponseCode::NoError;
                response.add_queries(query_msg.queries.clone());

                let record = hickory_proto::rr::Record::from_rdata(
                    query.name().clone(),
                    300,
                    hickory_proto::rr::RData::A(hickory_proto::rr::rdata::A(ip)),
                );
                response.add_answer(record);

                if let Ok(wire) = response.to_bytes() {
                    return Ok(wire);
                }
            }
        }

        // 2. Fast zero-dependency fallback
        if query_packet.len() < 12 {
            return Err(DnsError::EndOfBuffer);
        }

        let mut out = BytePacketBuffer::new();
        // ID
        out.write_u8(query_packet[0])?;
        out.write_u8(query_packet[1])?;

        // Flags: QR=1 (response), AA=1, RA=1, RCODE=0 (NoError)
        out.write_u16(0x8180)?;

        // QDCOUNT=1, ANCOUNT=1, NSCOUNT=0, ARCOUNT=0
        out.write_u16(1)?;
        out.write_u16(1)?;
        out.write_u16(0)?;
        out.write_u16(0)?;

        // Copy question section
        let mut in_buf = BytePacketBuffer::from_bytes(query_packet);
        in_buf.seek(12)?;
        let mut qname = String::new();
        in_buf.read_qname(&mut qname)?;
        let qtype = in_buf.read_u16()?;
        let qclass = in_buf.read_u16()?;

        out.write_qname(&qname)?;
        out.write_u16(qtype)?;
        out.write_u16(qclass)?;

        // Answer Section:
        out.write_u16(0xc00c)?; // Pointer to Question QNAME
        out.write_u16(1)?; // TYPE: A
        out.write_u16(1)?; // CLASS: IN
        out.write_u32(300)?; // TTL: 300
        out.write_u16(4)?; // RDLENGTH: 4
        for octet in ip.octets() {
            out.write_u8(octet)?;
        }

        Ok(out.filled().to_vec())
    }

    /// Constructs an authoritative empty NOERROR response (ANCOUNT=0), forcing IPv4 fallback for AAAA queries.
    pub fn build_empty_noerror_response(query_packet: &[u8]) -> Result<Vec<u8>, DnsError> {
        let parsed = Message::from_bytes(query_packet).ok();
        Self::build_empty_reply(
            query_packet,
            parsed.as_ref(),
            ResponseCode::NoError,
            true,
            0x8180,
        )
    }

    /// Constructs an RFC 1035 compliant SERVFAIL (Server Failure, RCODE=2) response.
    pub fn build_servfail_response(query_packet: &[u8]) -> Result<Vec<u8>, DnsError> {
        let parsed = Message::from_bytes(query_packet).ok();
        Self::build_empty_reply(
            query_packet,
            parsed.as_ref(),
            ResponseCode::ServFail,
            false,
            0x8182,
        )
    }

    /// Runs the UDP DNS proxy event loop.
    pub async fn run_udp_listener(&self) -> Result<(), DnsError> {
        let socket = Arc::new(UdpSocket::bind(&self.bind_addr).await?);
        info!("NUMA-style DNS proxy bound to UDP {}", self.bind_addr);

        let mut buf = [0u8; 4096];

        loop {
            let (len, client_addr) = match socket.recv_from(&mut buf).await {
                Ok(res) => res,
                Err(e) => {
                    // On Windows, a previous send_to that resulted in an ICMP Port Unreachable
                    // causes Winsock to return WSAECONNRESET (10054) on the subsequent recv_from.
                    // This is transient, non-fatal, and does not warrant an artificial 25ms delay or WARN spam.
                    if e.raw_os_error() == Some(10054) {
                        tracing::trace!("UDP recv transient WSAECONNRESET (10054) ignored");
                        continue;
                    }
                    warn!("UDP recv error: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_millis(25)).await;
                    continue;
                }
            };

            let query_wire = buf[..len].to_vec();
            if query_wire.len() < 12 {
                continue;
            }

            let socket = socket.clone();
            let blocklist = self.blocklist.clone();
            let cache = self.cache.clone();
            let log_tx = self.log_tx.clone();
            let balancer = self.balancer.clone();
            let socket_pool = self.socket_pool.clone();
            let in_flight = self.in_flight.clone();
            let program_lookup = self.program_lookup.clone();
            let stats = self.stats.clone();

            tokio::spawn(async move {
                Self::handle_query(
                    query_wire,
                    client_addr,
                    balancer,
                    socket,
                    blocklist,
                    cache,
                    log_tx,
                    socket_pool,
                    in_flight,
                    program_lookup,
                    stats,
                )
                .await;
            });
        }
    }

    async fn forward_with_pool(
        query_wire: &[u8],
        balancer: &NumaDnsBalancer,
        socket_pool: &UpstreamSocketPool,
    ) -> Result<Vec<u8>, DnsError> {
        let candidates = balancer.pick_ordered_upstreams();
        let req_txid = [query_wire[0], query_wire[1]];

        for (idx, metric) in candidates.iter().enumerate() {
            let target = metric.addr;
            let timeout_ms = if idx == 0 { 1500 } else { 2000 };
            let start_t = Instant::now();

            let upstream_sock = match socket_pool.acquire().await {
                Ok(s) => s,
                Err(_) => {
                    metric.record_failure();
                    continue;
                }
            };

            let query_fut = async {
                upstream_sock.send_to(query_wire, target).await?;
                let mut resp_buf = [0u8; 4096];
                let deadline =
                    tokio::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);
                loop {
                    let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
                    if remaining.is_zero() {
                        return Err(DnsError::EndOfBuffer);
                    }
                    let (resp_len, from) =
                        tokio::time::timeout(remaining, upstream_sock.recv_from(&mut resp_buf))
                            .await
                            .map_err(|_| DnsError::EndOfBuffer)??;
                    if from == target
                        && resp_len >= 2
                        && resp_buf[0] == req_txid[0]
                        && resp_buf[1] == req_txid[1]
                    {
                        return Ok::<Vec<u8>, DnsError>(resp_buf[..resp_len].to_vec());
                    }
                }
            };

            match query_fut.await {
                Ok(resp) => {
                    socket_pool.release(upstream_sock).await;
                    metric.record_success(start_t.elapsed());
                    return Ok(resp);
                }
                Err(e) => {
                    // Drop potentially dirty/unhealthy socket rather than returning to pool
                    metric.record_failure();
                    debug!(
                        "Upstream {} query failed/timeout: {}, attempting next...",
                        target, e
                    );
                }
            }
        }

        Err(DnsError::EndOfBuffer)
    }

    #[allow(clippy::too_many_arguments)]
    async fn handle_query(
        query_wire: Vec<u8>,
        client_addr: SocketAddr,
        balancer: Arc<NumaDnsBalancer>,
        socket: Arc<UdpSocket>,
        blocklist: Arc<BlocklistStore>,
        cache: Arc<DnsCache>,
        log_tx: Option<tokio::sync::mpsc::UnboundedSender<DnsLogEntry>>,
        socket_pool: UpstreamSocketPool,
        in_flight: InFlightMap,
        program_lookup: PortProgramLookup,
        stats: Arc<DnsStats>,
    ) {
        stats.queries_total.fetch_add(1, Ordering::Relaxed);
        // Parse once and reuse across question extraction & synthetic responses
        let parsed_msg = Message::from_bytes(&query_wire).ok();
        let (qname, qtype): (String, u16) = if let Some(ref msg) = parsed_msg {
            if let Some(query) = msg.queries.first() {
                let name = query.name().to_utf8().trim_end_matches('.').to_lowercase();
                let qt = u16::from(query.query_type());
                (name, qt)
            } else {
                return;
            }
        } else {
            let mut parser = BytePacketBuffer::from_bytes(&query_wire);
            if parser.seek(12).is_err() {
                return;
            }
            let mut name = String::new();
            if parser.read_qname(&mut name).is_err() {
                return;
            }
            let qt = parser.read_u16().unwrap_or(1);
            (name, qt)
        };

        let qtype_str = qtype_to_string(qtype);
        let process_name = if client_addr.ip().is_loopback() {
            program_lookup.lookup(client_addr.port())
        } else {
            None
        };

        // 1. Blocklist inspection (Atomic ArcSwap)
        let matched_local_rule = blocklist.is_blocked(&qname);
        if matched_local_rule {
            if blocklist.is_monitor_mode() {
                info!(
                    "[MONITOR MODE] Query for blacklisted domain '{}' observed; passing through without blocking",
                    qname
                );
            } else {
                stats.queries_blocked.fetch_add(1, Ordering::Relaxed);
                debug!("Blocked DNS request for {}", qname);
                if let Ok(blocked_wire) = Self::build_empty_reply(
                    &query_wire,
                    parsed_msg.as_ref(),
                    ResponseCode::NXDomain,
                    true,
                    0x8183,
                ) {
                    let _ = socket.send_to(&blocked_wire, client_addr).await;
                }
                if let Some(tx) = &log_tx {
                    let _ = tx.send(DnsLogEntry {
                        domain: qname.clone(),
                        qtype: qtype_str.to_string(),
                        action: "BLOCK".to_string(),
                        reason: "local_domain_rule".to_string(),
                        rule_id: Some(qname.clone()),
                        process_name,
                    });
                }
                return;
            }
        }

        // 1.5. SafeSearch & YouTube Restricted Mode Enforcer
        if let Some(cat) = crate::safesearch::match_safesearch_domain(&qname) {
            let active = match cat {
                crate::safesearch::SafeSearchCategory::YouTube => blocklist.is_youtube_restricted(),
                _ => blocklist.is_safesearch_enabled(),
            };

            if active && !blocklist.is_monitor_mode() {
                let reason = match cat {
                    crate::safesearch::SafeSearchCategory::YouTube => "youtube_restricted",
                    _ => "safesearch_enforced",
                };

                let resp_opt = match qtype {
                    1 => {
                        // A query -> Return SafeSearch VIP
                        Self::build_a_record_response_with_msg(
                            &query_wire,
                            parsed_msg.as_ref(),
                            cat.vip_ipv4(),
                        )
                        .ok()
                    }
                    _ => {
                        // AAAA (28), HTTPS (65), etc. -> Return empty NOERROR to force IPv4 VIP fallback
                        Self::build_empty_reply(
                            &query_wire,
                            parsed_msg.as_ref(),
                            ResponseCode::NoError,
                            true,
                            0x8180,
                        )
                        .ok()
                    }
                };

                if let Some(resp_wire) = resp_opt {
                    stats.queries_rewritten.fetch_add(1, Ordering::Relaxed);
                    debug!(
                        "Enforcing SafeSearch VIP rewrite for '{}' -> {}",
                        qname,
                        cat.display_target()
                    );
                    let _ = socket.send_to(&resp_wire, client_addr).await;
                    if let Some(tx) = &log_tx {
                        let _ = tx.send(DnsLogEntry {
                            domain: qname.clone(),
                            qtype: qtype_str.to_string(),
                            action: "REWRITE".to_string(),
                            reason: reason.to_string(),
                            rule_id: Some(cat.display_target().to_string()),
                            process_name,
                        });
                    }
                    return;
                }
            }
        }

        // 2. Serve-Stale & Prefetch Cache check
        let cache_key = format!("{}:{}", qname, qtype);
        let now = Instant::now();
        let mut served_stale = false;
        if let Some((cached_wire, is_stale, needs_prefetch)) =
            cache.get_with_prefetch(&cache_key, now)
        {
            if is_stale {
                stats.queries_stale.fetch_add(1, Ordering::Relaxed);
            } else {
                stats.queries_cached.fetch_add(1, Ordering::Relaxed);
            }
            let mut reply = cached_wire;
            if reply.len() >= 2 {
                // Patch transaction ID to match current client
                reply[0] = query_wire[0];
                reply[1] = query_wire[1];
            }
            let _ = socket.send_to(&reply, client_addr).await;
            if let Some(tx) = &log_tx {
                let _ = tx.send(DnsLogEntry {
                    domain: qname.clone(),
                    qtype: qtype_str.to_string(),
                    action: if matched_local_rule {
                        "MONITOR".to_string()
                    } else {
                        "ALLOW".to_string()
                    },
                    reason: if matched_local_rule {
                        "local_domain_rule".to_string()
                    } else if is_stale {
                        "stale_cache".to_string()
                    } else {
                        "cache".to_string()
                    },
                    rule_id: None,
                    process_name: process_name.clone(),
                });
            }
            if !is_stale {
                if needs_prefetch {
                    Self::spawn_prefetch(
                        cache_key,
                        query_wire,
                        balancer,
                        cache,
                        socket_pool,
                        in_flight,
                    );
                }
                return;
            }
            served_stale = true;
        }

        // 3. Single-flight deduplication for concurrent identical cache misses
        let (is_leader, notify) = {
            let mut map = in_flight.lock().await;
            if let Some(existing) = map.get(&cache_key) {
                (false, Arc::clone(existing))
            } else {
                let n = Arc::new(Notify::new());
                map.insert(cache_key.clone(), Arc::clone(&n));
                (true, n)
            }
        };

        if !is_leader && !served_stale {
            let _ = tokio::time::timeout(std::time::Duration::from_millis(1600), notify.notified())
                .await;
            if let Some((cached_wire, _)) = cache.get(&cache_key, Instant::now()) {
                stats.queries_coalesced.fetch_add(1, Ordering::Relaxed);
                let mut reply = cached_wire;
                if reply.len() >= 2 {
                    reply[0] = query_wire[0];
                    reply[1] = query_wire[1];
                }
                let _ = socket.send_to(&reply, client_addr).await;
                if let Some(tx) = &log_tx {
                    let _ = tx.send(DnsLogEntry {
                        domain: qname.clone(),
                        qtype: qtype_str.to_string(),
                        action: if matched_local_rule {
                            "MONITOR".to_string()
                        } else {
                            "ALLOW".to_string()
                        },
                        reason: if matched_local_rule {
                            "local_domain_rule".to_string()
                        } else {
                            "coalesced_cache".to_string()
                        },
                        rule_id: None,
                        process_name,
                    });
                }
                return;
            }
        }

        // 4. Upstream Forwarding with Dynamic NUMA Learning Balancer & Pooled UDP Socket
        let forward_result = Self::forward_with_pool(&query_wire, &balancer, &socket_pool).await;

        if is_leader {
            if let Ok(ref resp_wire) = forward_result {
                let ttl = DnsCache::extract_min_ttl(resp_wire);
                cache.insert(cache_key.clone(), resp_wire.clone(), ttl, Instant::now());
            }
            let mut map = in_flight.lock().await;
            map.remove(&cache_key);
            notify.notify_waiters();
        } else if let Ok(ref resp_wire) = forward_result {
            let ttl = DnsCache::extract_min_ttl(resp_wire);
            cache.insert(cache_key, resp_wire.clone(), ttl, Instant::now());
        }

        match forward_result {
            Ok(resp_wire) => {
                // A stale response was already returned and logged. Revalidation only
                // refreshes the cache; it must not emit a duplicate DNS response/log.
                if !served_stale {
                    stats.queries_upstream.fetch_add(1, Ordering::Relaxed);
                    let _ = socket.send_to(&resp_wire, client_addr).await;
                    if let Some(tx) = &log_tx {
                        let _ = tx.send(DnsLogEntry {
                            domain: qname.clone(),
                            qtype: qtype_str.to_string(),
                            action: if matched_local_rule {
                                "MONITOR".to_string()
                            } else {
                                "ALLOW".to_string()
                            },
                            reason: if matched_local_rule {
                                "local_domain_rule".to_string()
                            } else {
                                "upstream".to_string()
                            },
                            rule_id: matched_local_rule.then_some(qname.clone()),
                            process_name,
                        });
                    }
                }
            }
            Err(e) => {
                debug!("Failed to resolve {} upstream: {}", qname, e);
                if !served_stale {
                    stats.queries_error.fetch_add(1, Ordering::Relaxed);
                    if let Ok(fail_wire) = Self::build_empty_reply(
                        &query_wire,
                        parsed_msg.as_ref(),
                        ResponseCode::ServFail,
                        false,
                        0x8182,
                    ) {
                        let _ = socket.send_to(&fail_wire, client_addr).await;
                    }
                    if let Some(tx) = &log_tx {
                        let _ = tx.send(DnsLogEntry {
                            domain: qname,
                            qtype: qtype_str.to_string(),
                            action: "ERROR".to_string(),
                            reason: "upstream_failure".to_string(),
                            rule_id: None,
                            process_name,
                        });
                    }
                }
            }
        }
    }

    fn spawn_prefetch(
        cache_key: String,
        query_wire: Vec<u8>,
        balancer: Arc<NumaDnsBalancer>,
        cache: Arc<DnsCache>,
        socket_pool: UpstreamSocketPool,
        in_flight: InFlightMap,
    ) {
        tokio::spawn(async move {
            let notify = {
                let mut map = in_flight.lock().await;
                if map.contains_key(&cache_key) {
                    return;
                }
                let n = Arc::new(Notify::new());
                map.insert(cache_key.clone(), Arc::clone(&n));
                n
            };

            if let Ok(resp_wire) =
                Self::forward_with_pool(&query_wire, &balancer, &socket_pool).await
            {
                let ttl = DnsCache::extract_min_ttl(&resp_wire);
                cache.insert(cache_key.clone(), resp_wire, ttl, Instant::now());
            }

            let mut map = in_flight.lock().await;
            map.remove(&cache_key);
            notify.notify_waiters();
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hickory_proto::op::{MessageType, OpCode, Query};
    use hickory_proto::rr::{Name, RecordType};
    use std::str::FromStr;

    #[test]
    fn test_build_blocked_response_hickory() {
        let mut query_msg = Message::new(0x1234, MessageType::Query, OpCode::Query);
        let name = Name::from_str("blocked-domain.com.").unwrap();
        let query = Query::query(name, RecordType::A);
        query_msg.add_query(query);

        let query_wire = query_msg.to_bytes().unwrap();
        let resp_wire = DnsProxy::build_blocked_response(&query_wire).unwrap();

        let resp_msg = Message::from_bytes(&resp_wire).unwrap();
        assert_eq!(resp_msg.metadata.id, 0x1234);
        assert_eq!(resp_msg.metadata.message_type, MessageType::Response);
        assert_eq!(resp_msg.metadata.response_code, ResponseCode::NXDomain);
        assert!(resp_msg.metadata.authoritative);
        assert_eq!(resp_msg.queries.len(), 1);
        assert_eq!(
            resp_msg.queries[0].name().to_string(),
            "blocked-domain.com."
        );
    }

    #[test]
    fn test_build_servfail_response_hickory() {
        let mut query_msg = Message::new(0x4321, MessageType::Query, OpCode::Query);
        let name = Name::from_str("fail-domain.com.").unwrap();
        let query = Query::query(name, RecordType::A);
        query_msg.add_query(query);

        let query_wire = query_msg.to_bytes().unwrap();
        let resp_wire = DnsProxy::build_servfail_response(&query_wire).unwrap();

        let resp_msg = Message::from_bytes(&resp_wire).unwrap();
        assert_eq!(resp_msg.metadata.id, 0x4321);
        assert_eq!(resp_msg.metadata.message_type, MessageType::Response);
        assert_eq!(resp_msg.metadata.response_code, ResponseCode::ServFail);
        assert_eq!(resp_msg.queries.len(), 1);
        assert_eq!(resp_msg.queries[0].name().to_string(), "fail-domain.com.");
    }

    #[test]
    fn test_build_a_record_response_hickory() {
        use hickory_proto::rr::rdata;
        let mut query_msg = Message::new(0x5678, MessageType::Query, OpCode::Query);
        let name = Name::from_str("www.google.com.").unwrap();
        let query = Query::query(name, RecordType::A);
        query_msg.add_query(query);

        let query_wire = query_msg.to_bytes().unwrap();
        let vip = std::net::Ipv4Addr::new(216, 239, 38, 120);
        let resp_wire = DnsProxy::build_a_record_response(&query_wire, vip).unwrap();

        let resp_msg = Message::from_bytes(&resp_wire).unwrap();
        assert_eq!(resp_msg.metadata.id, 0x5678);
        assert_eq!(resp_msg.metadata.message_type, MessageType::Response);
        assert_eq!(resp_msg.metadata.response_code, ResponseCode::NoError);
        assert_eq!(resp_msg.answers.len(), 1);
        if let hickory_proto::rr::RData::A(rdata::A(ip)) = &resp_msg.answers[0].data {
            assert_eq!(*ip, vip);
        } else {
            panic!("Expected A record in answer");
        }
    }

    #[test]
    fn test_build_empty_noerror_response() {
        let mut query_msg = Message::new(0x9999, MessageType::Query, OpCode::Query);
        let name = Name::from_str("www.google.com.").unwrap();
        let query = Query::query(name, RecordType::AAAA);
        query_msg.add_query(query);

        let query_wire = query_msg.to_bytes().unwrap();
        let resp_wire = DnsProxy::build_empty_noerror_response(&query_wire).unwrap();

        let resp_msg = Message::from_bytes(&resp_wire).unwrap();
        assert_eq!(resp_msg.metadata.id, 0x9999);
        assert_eq!(resp_msg.metadata.response_code, ResponseCode::NoError);
        assert_eq!(resp_msg.answers.len(), 0);
    }
}

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::net::UdpSocket;
use tracing::{debug, info, warn};

use crate::blocklist::BlocklistStore;
use crate::buffer::BytePacketBuffer;
use crate::cache::DnsCache;
use crate::error::DnsError;

pub const DEFAULT_BIND_ADDR: &str = "127.0.0.2:53";
pub const DEFAULT_UPSTREAM: &str = "1.1.1.1:53";

pub struct DnsProxy {
    bind_addr: String,
    upstream_addr: String,
    blocklist: Arc<BlocklistStore>,
    cache: Arc<DnsCache>,
}

impl DnsProxy {
    pub fn new(
        bind_addr: impl Into<String>,
        upstream_addr: impl Into<String>,
        blocklist: Arc<BlocklistStore>,
        cache: Arc<DnsCache>,
    ) -> Self {
        Self {
            bind_addr: bind_addr.into(),
            upstream_addr: upstream_addr.into(),
            blocklist,
            cache,
        }
    }

    /// Builds a synthetic NXDOMAIN or 0.0.0.0 blocked DNS response wire.
    pub fn build_blocked_response(query_packet: &[u8]) -> Result<Vec<u8>, DnsError> {
        if query_packet.len() < 12 {
            return Err(DnsError::EndOfBuffer);
        }

        let mut out = BytePacketBuffer::new();
        // ID (match query ID)
        out.write_u8(query_packet[0])?;
        out.write_u8(query_packet[1])?;

        // Flags: QR=1 (response), AA=1, RA=1, RCODE=3 (NXDOMAIN)
        // 0x8183 = 1000 0001 1000 0011
        out.write_u16(0x8183)?;

        // QDCOUNT=1, ANCOUNT=0, NSCOUNT=0, ARCOUNT=0
        out.write_u16(1)?;
        out.write_u16(0)?;
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

        Ok(out.filled().to_vec())
    }

    /// Runs the UDP DNS proxy event loop.
    pub async fn run_udp_listener(&self) -> Result<(), DnsError> {
        let socket = UdpSocket::bind(&self.bind_addr).await?;
        info!("NUMA-style DNS proxy bound to UDP {}", self.bind_addr);

        let upstream_sock = UdpSocket::bind("0.0.0.0:0").await?;
        let upstream_target: SocketAddr = self.upstream_addr.parse().map_err(|e| {
            DnsError::Format(format!(
                "Invalid upstream address {}: {}",
                self.upstream_addr, e
            ))
        })?;

        let mut buf = [0u8; 4096];

        loop {
            let (len, client_addr) = match socket.recv_from(&mut buf).await {
                Ok(res) => res,
                Err(e) => {
                    warn!("UDP recv error: {}", e);
                    continue;
                }
            };

            let query_wire = &buf[..len];
            if query_wire.len() < 12 {
                continue;
            }

            // Extract QNAME
            let mut parser = BytePacketBuffer::from_bytes(query_wire);
            if parser.seek(12).is_err() {
                continue;
            }

            let mut qname = String::new();
            if parser.read_qname(&mut qname).is_err() {
                continue;
            }
            let qtype = parser.read_u16().unwrap_or(1);

            // 1. Blocklist inspection (Atomic ArcSwap)
            if self.blocklist.is_blocked(&qname) {
                if self.blocklist.is_monitor_mode() {
                    info!(
                        "[MONITOR MODE] Query for blacklisted domain '{}' observed; passing through without blocking",
                        qname
                    );
                } else {
                    debug!("Blocked DNS request for {}", qname);
                    if let Ok(blocked_wire) = Self::build_blocked_response(query_wire) {
                        let _ = socket.send_to(&blocked_wire, client_addr).await;
                    }
                    continue;
                }
            }

            // 2. Serve-Stale Cache check
            let cache_key = format!("{}:{}", qname, qtype);
            let now = Instant::now();
            if let Some((cached_wire, is_stale)) = self.cache.get(&cache_key, now) {
                let mut reply = cached_wire;
                // Patch transaction ID to match current client
                reply[0] = query_wire[0];
                reply[1] = query_wire[1];
                let _ = socket.send_to(&reply, client_addr).await;
                if !is_stale {
                    continue;
                }
            }

            // 3. Upstream Forwarding
            if upstream_sock
                .send_to(query_wire, upstream_target)
                .await
                .is_ok()
            {
                let mut resp_buf = [0u8; 4096];
                if let Ok(resp_len) = upstream_sock.recv(&mut resp_buf).await {
                    let resp_wire = &resp_buf[..resp_len];
                    // Cache response with 300s default TTL
                    self.cache.insert(cache_key, resp_wire.to_vec(), 300, now);

                    let _ = socket.send_to(resp_wire, client_addr).await;
                }
            }
        }
    }
}

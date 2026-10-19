use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::net::UdpSocket;
use tracing::{debug, info, warn};

use crate::blocklist::BlocklistStore;
use crate::buffer::BytePacketBuffer;
use crate::cache::DnsCache;
use crate::error::DnsError;
use hickory_proto::op::{Message, MessageType, ResponseCode};
use hickory_proto::serialize::binary::{BinDecodable, BinEncodable};

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

    /// Builds a synthetic NXDOMAIN or 0.0.0.0 blocked DNS response wire using hickory-proto.
    pub fn build_blocked_response(query_packet: &[u8]) -> Result<Vec<u8>, DnsError> {
        // 1. Attempt RFC-compliant response generation via hickory-proto
        if let Ok(query_msg) = Message::from_bytes(query_packet) {
            let mut response = Message::new(
                query_msg.metadata.id,
                MessageType::Response,
                query_msg.metadata.op_code,
            );
            response.metadata.recursion_desired = query_msg.metadata.recursion_desired;
            response.metadata.authoritative = true;
            response.metadata.recursion_available = true;
            response.metadata.response_code = ResponseCode::NXDomain;

            response.add_queries(query_msg.queries.clone());

            if let Ok(wire) = response.to_bytes() {
                return Ok(wire);
            }
        }

        // 2. Fast zero-dependency fallback
        if query_packet.len() < 12 {
            return Err(DnsError::EndOfBuffer);
        }

        let mut out = BytePacketBuffer::new();
        // ID (match query ID)
        out.write_u8(query_packet[0])?;
        out.write_u8(query_packet[1])?;

        // Flags: QR=1 (response), AA=1, RA=1, RCODE=3 (NXDOMAIN)
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

            // Extract QNAME and QTYPE using hickory-proto with fallback
            let (qname, qtype): (String, u16) = if let Ok(msg) = Message::from_bytes(query_wire) {
                if let Some(query) = msg.queries.first() {
                    let name = query.name().to_utf8().trim_end_matches('.').to_lowercase();
                    let qt = u16::from(query.query_type());
                    (name, qt)
                } else {
                    continue;
                }
            } else {
                let mut parser = BytePacketBuffer::from_bytes(query_wire);
                if parser.seek(12).is_err() {
                    continue;
                }
                let mut name = String::new();
                if parser.read_qname(&mut name).is_err() {
                    continue;
                }
                let qt = parser.read_u16().unwrap_or(1);
                (name, qt)
            };

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
}

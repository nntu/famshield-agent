use arc_swap::ArcSwap;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Cacheline aligned upstream metrics to prevent false-sharing across CPU cores / NUMA nodes.
#[repr(align(64))]
pub struct UpstreamMetrics {
    pub addr: SocketAddr,
    pub original_target: String,
    pub ewma_rtt_us: AtomicU64,
    pub total_queries: AtomicU64,
    pub success_count: AtomicU64,
    pub failure_count: AtomicU64,
    pub consecutive_fails: AtomicU32,
}

impl UpstreamMetrics {
    pub fn new(addr: SocketAddr, original_target: String) -> Self {
        Self {
            addr,
            original_target,
            ewma_rtt_us: AtomicU64::new(40_000), // Default baseline 40ms
            total_queries: AtomicU64::new(0),
            success_count: AtomicU64::new(0),
            failure_count: AtomicU64::new(0),
            consecutive_fails: AtomicU32::new(0),
        }
    }

    /// Dynamic score combining EWMA latency and failure penalty.
    /// Lower score represents higher priority / optimal DNS server.
    pub fn effective_score(&self) -> u64 {
        let rtt = self.ewma_rtt_us.load(Ordering::Relaxed);
        let fails = self.consecutive_fails.load(Ordering::Relaxed);
        let penalty_us = (fails as u64) * 250_000; // 250ms penalty per consecutive failure
        rtt.saturating_add(penalty_us)
    }

    /// Update EWMA with newly observed round-trip duration.
    pub fn record_success(&self, duration: Duration) {
        let rtt_us = duration.as_micros() as u64;
        let old = self.ewma_rtt_us.load(Ordering::Relaxed);
        let new_ewma = if old == 0 {
            rtt_us
        } else {
            // Adaptive EWMA: alpha = 0.25 (75% history + 25% new sample)
            ((old * 3) + rtt_us) / 4
        };
        self.ewma_rtt_us.store(new_ewma, Ordering::Relaxed);
        self.consecutive_fails.store(0, Ordering::Relaxed);
        self.success_count.fetch_add(1, Ordering::Relaxed);
        self.total_queries.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a failed or timed out query, adding exponential penalty.
    pub fn record_failure(&self) {
        self.failure_count.fetch_add(1, Ordering::Relaxed);
        let fails = self.consecutive_fails.fetch_add(1, Ordering::Relaxed) + 1;
        self.total_queries.fetch_add(1, Ordering::Relaxed);
        let old = self.ewma_rtt_us.load(Ordering::Relaxed);
        // Exponentially increase penalty up to 5 seconds
        let penalty = 150_000u64.saturating_mul(fails as u64);
        self.ewma_rtt_us.store(
            old.saturating_add(penalty).min(5_000_000),
            Ordering::Relaxed,
        );
    }

    pub fn latency_ms(&self) -> f64 {
        (self.ewma_rtt_us.load(Ordering::Relaxed) as f64) / 1000.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpstreamDiagnostic {
    pub target: String,
    pub addr: String,
    pub latency_ms: f64,
    pub total_queries: u64,
    pub success_count: u64,
    pub failure_count: u64,
    pub consecutive_fails: u32,
    pub is_optimal: bool,
}

/// NUMA-inspired, lock-free dynamic learning load balancer for DNS upstream servers.
pub struct NumaDnsBalancer {
    upstreams: ArcSwap<Vec<Arc<UpstreamMetrics>>>,
    query_counter: AtomicU64,
}

impl NumaDnsBalancer {
    pub fn new(targets: &[String]) -> Self {
        let metrics = Self::parse_targets(targets, &[]);
        Self {
            upstreams: ArcSwap::from_pointee(metrics),
            query_counter: AtomicU64::new(0),
        }
    }

    fn parse_targets(
        targets: &[String],
        existing: &[Arc<UpstreamMetrics>],
    ) -> Vec<Arc<UpstreamMetrics>> {
        let mut result = Vec::new();
        for t in targets {
            let t_trimmed = t.trim();
            if t_trimmed.is_empty() {
                continue;
            }
            let parsed_addr: Result<SocketAddr, _> = if t_trimmed.contains(':') {
                t_trimmed.parse()
            } else {
                format!("{}:53", t_trimmed).parse()
            };

            if let Ok(addr) = parsed_addr {
                // If we already had metrics for this address, retain them to preserve learned latency
                if let Some(prev) = existing.iter().find(|e| e.addr == addr) {
                    result.push(prev.clone());
                } else {
                    result.push(Arc::new(UpstreamMetrics::new(addr, t_trimmed.to_string())));
                }
            }
        }

        // Fallback default if empty
        if result.is_empty() {
            if let Ok(addr) = "1.1.1.1:53".parse() {
                result.push(Arc::new(UpstreamMetrics::new(addr, "1.1.1.1".to_string())));
            }
        }

        result
    }

    /// Update upstream servers dynamically while preserving historical learning metrics.
    pub fn set_upstreams(&self, targets: &[String]) {
        let current = self.upstreams.load();
        let updated = Self::parse_targets(targets, &current);
        self.upstreams.store(Arc::new(updated));
    }

    /// Returns upstream candidates ordered from optimal (lowest effective latency) to least preferred.
    /// Employs epsilon-exploration (1 in every 25 queries) to probe alternative servers,
    /// enabling continuous learning and adaptive self-healing.
    pub fn pick_ordered_upstreams(&self) -> Vec<Arc<UpstreamMetrics>> {
        let list = (**self.upstreams.load()).clone();
        if list.len() <= 1 {
            return list;
        }

        let mut ranked = list;
        ranked.sort_by_key(|u| u.effective_score());

        let q_idx = self.query_counter.fetch_add(1, Ordering::Relaxed);
        // Epsilon-exploration: probe secondary upstream every 25 queries to discover faster routes
        if q_idx.is_multiple_of(25) && ranked.len() > 1 {
            ranked.swap(0, 1);
        }

        ranked
    }

    /// Returns the currently optimal DNS server according to learned metrics.
    pub fn get_optimal_upstream(&self) -> Option<Arc<UpstreamMetrics>> {
        let list = self.upstreams.load();
        list.iter().min_by_key(|u| u.effective_score()).cloned()
    }

    /// Returns diagnostic information for all configured upstreams.
    pub fn get_diagnostics(&self) -> Vec<UpstreamDiagnostic> {
        let list = self.upstreams.load();
        let optimal_addr = list
            .iter()
            .min_by_key(|u| u.effective_score())
            .map(|u| u.addr);

        list.iter()
            .map(|u| UpstreamDiagnostic {
                target: u.original_target.clone(),
                addr: u.addr.to_string(),
                latency_ms: (u.latency_ms() * 10.0).round() / 10.0,
                total_queries: u.total_queries.load(Ordering::Relaxed),
                success_count: u.success_count.load(Ordering::Relaxed),
                failure_count: u.failure_count.load(Ordering::Relaxed),
                consecutive_fails: u.consecutive_fails.load(Ordering::Relaxed),
                is_optimal: Some(u.addr) == optimal_addr,
            })
            .collect()
    }
}

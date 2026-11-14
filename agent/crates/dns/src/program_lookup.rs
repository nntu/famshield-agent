use listeners::Protocol;
use std::collections::HashMap;
use std::sync::mpsc::{self, SyncSender, TrySendError};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

const RETRY_TIMEOUT: Duration = Duration::from_millis(1500);
const VALID_PROGRAM_TIMEOUT: Duration = Duration::from_secs(60);
const MAX_PORT_CACHE_ENTRIES: usize = 1024;
const LOOKUP_QUEUE_CAPACITY: usize = 256;

#[derive(Clone, Debug)]
struct LookedUpPortProgram {
    process_name: Option<String>,
    updated_at: Instant,
    retried: bool,
}

/// Non-blocking UDP port-to-process resolver inspired by Sniffnet's `ProgramLookup`.
///
/// Offloads `listeners::get_process_by_port` calls to a dedicated background thread
/// and caches results with TTL + retry cooldown so the DNS hot-path never stalls.
#[derive(Clone)]
pub struct PortProgramLookup {
    port_tx: SyncSender<u16>,
    cache: Arc<RwLock<HashMap<u16, LookedUpPortProgram>>>,
}

impl Default for PortProgramLookup {
    fn default() -> Self {
        Self::new()
    }
}

impl PortProgramLookup {
    pub fn new() -> Self {
        let (port_tx, port_rx) = mpsc::sync_channel::<u16>(LOOKUP_QUEUE_CAPACITY);
        let cache = Arc::new(RwLock::new(HashMap::<u16, LookedUpPortProgram>::new()));
        let worker_cache = Arc::clone(&cache);

        let _ = std::thread::Builder::new()
            .name("dns_program_lookup".to_string())
            .spawn(move || {
                while let Ok(port) = port_rx.recv() {
                    let process_name = listeners::get_process_by_port(port, Protocol::UDP)
                        .ok()
                        .map(|proc_info| proc_info.name)
                        .filter(|name| !name.trim().is_empty());

                    if let Ok(mut map) = worker_cache.write() {
                        if let Some(entry) = map.get_mut(&port) {
                            entry.process_name = process_name;
                            entry.updated_at = Instant::now();
                        } else {
                            if map.len() >= MAX_PORT_CACHE_ENTRIES {
                                prune_stale_entries(&mut map);
                            }
                            map.insert(
                                port,
                                LookedUpPortProgram {
                                    process_name,
                                    updated_at: Instant::now(),
                                    retried: false,
                                },
                            );
                        }
                    }
                }
            });

        Self { port_tx, cache }
    }

    /// Returns the cached process name for `port` if available and fresh,
    /// or schedules a background lookup without blocking the caller.
    pub fn lookup(&self, port: u16) -> Option<String> {
        if port == 0 {
            return None;
        }

        let now = Instant::now();
        // Fast read path
        if let Ok(map) = self.cache.read() {
            if let Some(entry) = map.get(&port) {
                let elapsed = now.duration_since(entry.updated_at);
                if entry.process_name.is_some() && elapsed < VALID_PROGRAM_TIMEOUT {
                    return entry.process_name.clone();
                }
                if entry.process_name.is_none()
                    && (elapsed < RETRY_TIMEOUT || entry.retried && elapsed < VALID_PROGRAM_TIMEOUT)
                {
                    return None;
                }
            }
        }

        // Slow write path: record intent and dispatch to background worker
        let mut should_send = false;
        let mut cached_result = None;
        if let Ok(mut map) = self.cache.write() {
            if map.len() >= MAX_PORT_CACHE_ENTRIES {
                prune_stale_entries(&mut map);
            }

            if let Some(entry) = map.get_mut(&port) {
                let elapsed = now.duration_since(entry.updated_at);
                if entry.process_name.is_some() {
                    if elapsed < VALID_PROGRAM_TIMEOUT {
                        cached_result = entry.process_name.clone();
                    } else {
                        entry.retried = false;
                        entry.updated_at = now;
                        should_send = true;
                    }
                } else if elapsed >= RETRY_TIMEOUT
                    && (!entry.retried || elapsed >= VALID_PROGRAM_TIMEOUT)
                {
                    entry.retried = elapsed < VALID_PROGRAM_TIMEOUT;
                    entry.updated_at = now;
                    should_send = true;
                }
            } else {
                map.insert(
                    port,
                    LookedUpPortProgram {
                        process_name: None,
                        updated_at: now,
                        retried: false,
                    },
                );
                should_send = true;
            }
        }

        if should_send {
            match self.port_tx.try_send(port) {
                Ok(()) | Err(TrySendError::Full(_)) | Err(TrySendError::Disconnected(_)) => {}
            }
        }

        cached_result
    }
}

fn prune_stale_entries(map: &mut HashMap<u16, LookedUpPortProgram>) {
    map.retain(|_, entry| entry.updated_at.elapsed() < VALID_PROGRAM_TIMEOUT);
    if map.len() >= MAX_PORT_CACHE_ENTRIES {
        let mut by_age: Vec<(u16, Instant)> = map.iter().map(|(&p, v)| (p, v.updated_at)).collect();
        by_age.sort_by_key(|(_, ts)| *ts);
        let to_remove = MAX_PORT_CACHE_ENTRIES / 4;
        for (port, _) in by_age.into_iter().take(to_remove) {
            map.remove(&port);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_program_lookup_non_blocking_and_zero_port() {
        let resolver = PortProgramLookup::new();
        assert_eq!(resolver.lookup(0), None);
        // Should not block or panic on an arbitrary ephemeral port
        let _ = resolver.lookup(54321);
    }

    #[test]
    fn test_port_program_lookup_returns_cached_entry() {
        let resolver = PortProgramLookup::new();
        if let Ok(mut map) = resolver.cache.write() {
            map.insert(
                49152,
                LookedUpPortProgram {
                    process_name: Some("chrome.exe".to_string()),
                    updated_at: Instant::now(),
                    retried: false,
                },
            );
        }
        assert_eq!(resolver.lookup(49152).as_deref(), Some("chrome.exe"));
    }
}

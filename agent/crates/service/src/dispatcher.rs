use chrono::Utc;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tracing::{debug, error, info};

use crate::ipc::IpcMessage;
use famshield_storage::{StorageEngine, VisualEvidenceRecord};

type ActiveAppSession = (String, chrono::DateTime<Utc>, i64);

pub struct IpcDispatcher {
    storage: StorageEngine,
    pending_captures: Arc<Mutex<VecDeque<(String, String, bool)>>>,
    recent_triggers: Arc<Mutex<HashMap<String, Instant>>>,
    active_tracker: Arc<Mutex<Option<ActiveAppSession>>>,
    telemetry_pipeline: Arc<Mutex<Option<Arc<famshield_sync::TelemetryPipeline>>>>,
}

impl IpcDispatcher {
    pub fn new(storage: StorageEngine) -> Self {
        Self {
            storage,
            pending_captures: Arc::new(Mutex::new(VecDeque::new())),
            recent_triggers: Arc::new(Mutex::new(HashMap::new())),
            active_tracker: Arc::new(Mutex::new(None)),
            telemetry_pipeline: Arc::new(Mutex::new(None)),
        }
    }

    pub fn set_telemetry(&self, telemetry: Arc<famshield_sync::TelemetryPipeline>) {
        if let Ok(mut guard) = self.telemetry_pipeline.lock() {
            *guard = Some(telemetry);
        }
    }

    /// Queues a visual evidence capture trigger (e.g. from malicious DNS block) with throttling.
    pub fn queue_capture_trigger(&self, event_id: String, trigger: String) -> bool {
        let is_silent = self
            .storage
            .get_meta("silent_mode")
            .unwrap_or(None)
            .map(|s| s == "true" || s == "1")
            .unwrap_or(false)
            || trigger.starts_with("WEB_BLOCK");
        self.queue_capture_trigger_with_options(event_id, trigger, is_silent)
    }

    /// Queues a visual evidence capture trigger with explicit silent control.
    pub fn queue_capture_trigger_with_options(
        &self,
        event_id: String,
        trigger: String,
        silent: bool,
    ) -> bool {
        let now = Instant::now();
        let mut recents = match self.recent_triggers.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };

        // Purge entries older than 30 seconds
        recents.retain(|_, time| now.duration_since(*time).as_secs() < 30);

        // Throttle identical trigger within 10 seconds to avoid capture spam
        if let Some(prev) = recents.get(&trigger) {
            if now.duration_since(*prev).as_secs() < 10 {
                return false;
            }
        }
        recents.insert(trigger.clone(), now);

        let mut queue = match self.pending_captures.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };

        if queue.len() < 5 {
            info!(
                "Queued visual evidence capture trigger: '{}' (silent: {})",
                trigger, silent
            );
            queue.push_back((event_id, trigger, silent));
            true
        } else {
            false
        }
    }

    /// Dispatches an incoming parsed IPC message from an authenticated session.
    pub async fn dispatch(&self, message: IpcMessage) -> IpcMessage {
        match message {
            IpcMessage::SessionHello {
                session_id,
                windows_username,
            } => {
                info!(
                    "Registered session {} for user '{}'",
                    session_id, windows_username
                );
                IpcMessage::Response {
                    request_id: "hello".into(),
                    success: true,
                    message: format!("Session {} registered", session_id),
                }
            }
            IpcMessage::ForegroundHeartbeat {
                session_id,
                executable_name,
                is_idle,
                idle_duration_ms: _,
            } => {
                debug!(
                    "Session {} foreground: exe='{}', idle={}",
                    session_id, executable_name, is_idle
                );

                // Accumulate application usage when user is actively interacting with foreground app
                if !is_idle && !executable_name.is_empty() && executable_name != "unknown" {
                    let mut tracker_guard = match self.active_tracker.lock() {
                        Ok(g) => g,
                        Err(p) => p.into_inner(),
                    };

                    let now = Utc::now();
                    if let Some((ref cur_app, ref start_time, ref mut dur_ms)) = *tracker_guard {
                        if cur_app.eq_ignore_ascii_case(&executable_name) {
                            *dur_ms += 2000;
                            // Checkpoint every 10 seconds to storage
                            if *dur_ms % 10_000 == 0 {
                                let seg_id = format!("seg-{}", start_time.timestamp_millis());
                                let _ = self.storage.record_usage_segment(
                                    &famshield_storage::UsageSegmentRecord {
                                        segment_id: seg_id,
                                        child_id: "local_child".to_string(),
                                        rule_id: executable_name.clone(),
                                        start_utc: *start_time,
                                        duration_ms: *dur_ms,
                                        state: "committed".to_string(),
                                    },
                                );
                            }
                        } else {
                            // Flushed previous app segment
                            let seg_id = format!("seg-{}", start_time.timestamp_millis());
                            let _ = self.storage.record_usage_segment(
                                &famshield_storage::UsageSegmentRecord {
                                    segment_id: seg_id,
                                    child_id: "local_child".to_string(),
                                    rule_id: cur_app.clone(),
                                    start_utc: *start_time,
                                    duration_ms: *dur_ms,
                                    state: "committed".to_string(),
                                },
                            );
                            if let Ok(guard) = self.telemetry_pipeline.lock() {
                                if let Some(ref tel) = *guard {
                                    let _ = tel.queue_app_usage(
                                        cur_app,
                                        *dur_ms,
                                        1,
                                        *start_time,
                                        now,
                                        None,
                                    );
                                }
                            }
                            *tracker_guard = Some((executable_name.clone(), now, 2000));
                        }
                    } else {
                        *tracker_guard = Some((executable_name.clone(), now, 2000));
                        let seg_id = format!("seg-{}", now.timestamp_millis());
                        let _ = self.storage.record_usage_segment(
                            &famshield_storage::UsageSegmentRecord {
                                segment_id: seg_id,
                                child_id: "local_child".to_string(),
                                rule_id: executable_name.clone(),
                                start_utc: now,
                                duration_ms: 2000,
                                state: "committed".to_string(),
                            },
                        );
                    }
                } else if is_idle {
                    let mut tracker_guard = match self.active_tracker.lock() {
                        Ok(g) => g,
                        Err(p) => p.into_inner(),
                    };
                    if let Some((cur_app, start_time, dur_ms)) = tracker_guard.take() {
                        let seg_id = format!("seg-{}", start_time.timestamp_millis());
                        let _ = self.storage.record_usage_segment(
                            &famshield_storage::UsageSegmentRecord {
                                segment_id: seg_id,
                                child_id: "local_child".to_string(),
                                rule_id: cur_app.clone(),
                                start_utc: start_time,
                                duration_ms: dur_ms,
                                state: "committed".to_string(),
                            },
                        );
                        if let Ok(guard) = self.telemetry_pipeline.lock() {
                            if let Some(ref tel) = *guard {
                                let _ = tel.queue_app_usage(
                                    &cur_app,
                                    dur_ms,
                                    1,
                                    start_time,
                                    Utc::now(),
                                    None,
                                );
                            }
                        }
                    }
                }

                // Check if any visual evidence capture command is pending for the session
                let next_capture = {
                    let mut queue = match self.pending_captures.lock() {
                        Ok(g) => g,
                        Err(p) => p.into_inner(),
                    };
                    queue.pop_front()
                };

                if let Some((event_id, trigger, silent)) = next_capture {
                    info!(
                        "Dispatching CaptureEvidenceCommand to session helper for trigger: '{}' (silent: {})",
                        trigger, silent
                    );
                    IpcMessage::CaptureEvidenceCommand {
                        event_id,
                        trigger,
                        silent,
                    }
                } else {
                    IpcMessage::Response {
                        request_id: "heartbeat".into(),
                        success: true,
                        message: "Acknowledged".into(),
                    }
                }
            }
            IpcMessage::VisualEvidenceSubmission {
                event_id,
                trigger,
                image_jpeg_bytes,
            } => {
                let size_bytes = image_jpeg_bytes.len();
                let now = Utc::now();
                let evidence_id = format!("evi-{}", uuid_simple());
                let record = VisualEvidenceRecord {
                    evidence_id: evidence_id.clone(),
                    event_id,
                    occurred_at: now,
                    trigger,
                    image_payload: image_jpeg_bytes,
                    size_bytes,
                    expires_at: now + chrono::Duration::hours(48), // 48-Hour Retention Limit
                };

                match self.storage.save_visual_evidence(&record) {
                    Ok(()) => {
                        info!(
                            "Stored visual evidence '{}' ({} bytes, expires in 48h)",
                            evidence_id, size_bytes
                        );
                        IpcMessage::Response {
                            request_id: evidence_id,
                            success: true,
                            message: "Evidence persisted".into(),
                        }
                    }
                    Err(e) => {
                        error!("Failed to save visual evidence: {}", e);
                        IpcMessage::Response {
                            request_id: "error".into(),
                            success: false,
                            message: format!("Storage error: {}", e),
                        }
                    }
                }
            }
            IpcMessage::TimeExtensionRequest {
                session_id,
                requested_minutes,
                reason,
            } => {
                info!(
                    "Session {} requested {} minutes: {}",
                    session_id, requested_minutes, reason
                );
                IpcMessage::Response {
                    request_id: "time_req".into(),
                    success: true,
                    message: "Time extension request recorded".into(),
                }
            }
            IpcMessage::KeywordViolation {
                session_id,
                app_name,
                keyword_matched,
                category,
                severity,
                snippet_masked,
            } => {
                info!(
                    "Session {} keyword violation: app='{}', keyword='{}', cat='{}', sev='{}'",
                    session_id, app_name, keyword_matched, category, severity
                );
                let record = famshield_storage::KeywordViolationRecord {
                    id: format!("kw_v_{}_{}", Utc::now().timestamp_micros(), session_id),
                    timestamp_utc: Utc::now(),
                    app_name,
                    keyword_matched,
                    category,
                    severity,
                    snippet_masked,
                };
                match self.storage.record_keyword_violation(&record) {
                    Ok(()) => IpcMessage::Response {
                        request_id: "kw_violation".into(),
                        success: true,
                        message: "Keyword violation recorded".into(),
                    },
                    Err(e) => {
                        error!("Failed to record keyword violation: {}", e);
                        IpcMessage::Response {
                            request_id: "error".into(),
                            success: false,
                            message: format!("Storage error: {}", e),
                        }
                    }
                }
            }
            IpcMessage::ReloadKeywordRules { enabled } => {
                info!("Reloading keyword rules: enabled={}", enabled);
                IpcMessage::Response {
                    request_id: "reload_kw".into(),
                    success: true,
                    message: format!("Keyword rules reloaded (enabled: {})", enabled),
                }
            }
            IpcMessage::GetKeywordRulesRequest => {
                let enabled = self
                    .storage
                    .get_meta("keyboard_monitoring_enabled")
                    .unwrap_or(None)
                    .map(|s| s != "false")
                    .unwrap_or(true);

                let mut rules = famshield_core::keyword::default_keyword_rules();
                // Apply disabled overrides from metadata
                for r in &mut rules {
                    let override_key = format!("kw_disabled_{}", r.id);
                    if self
                        .storage
                        .get_meta(&override_key)
                        .unwrap_or(None)
                        .as_deref()
                        == Some("1")
                    {
                        r.enabled = false;
                    }
                }
                // Append custom keyword rules from SQLite
                if let Ok(customs) = self.storage.list_custom_keywords() {
                    for c in customs {
                        rules.push(famshield_core::keyword::KeywordRule {
                            id: c.id,
                            pattern: c.pattern,
                            category: famshield_core::keyword::KeywordCategory::Custom,
                            severity: famshield_core::keyword::KeywordSeverity::Medium,
                            enabled: c.enabled,
                        });
                    }
                }

                let silent_mode = self
                    .storage
                    .get_meta("silent_mode")
                    .unwrap_or(None)
                    .map(|s| s == "true" || s == "1")
                    .unwrap_or(false);

                IpcMessage::GetKeywordRulesResponse {
                    enabled,
                    rules,
                    silent_mode,
                }
            }
            IpcMessage::GetKeywordRulesResponse { .. } => IpcMessage::Response {
                request_id: "none".into(),
                success: false,
                message: "Unexpected response message received".into(),
            },
            IpcMessage::CaptureEvidenceCommand { .. } => IpcMessage::Response {
                request_id: "none".into(),
                success: false,
                message: "CaptureEvidenceCommand is outbound only".into(),
            },
            IpcMessage::Response { .. } => IpcMessage::Response {
                request_id: "none".into(),
                success: false,
                message: "Unexpected response message received".into(),
            },
        }
    }
}

fn uuid_simple() -> String {
    use std::time::SystemTime;
    let since = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{:x}", since)
}

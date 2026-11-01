pub mod assets;
pub mod handlers;
pub mod types;

pub(crate) use assets::*;
pub(crate) use handlers::*;
pub use types::*;

use axum::{
    http::{header, HeaderMap},
    routing::{delete, get, post},
    Router,
};
use chrono::{DateTime, Duration, Utc};
use rand::RngCore;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tracing::{info, warn};

use crate::service::LocalAdminService;
use famshield_dns::blocklist::BlocklistStore;
use famshield_storage::UsageSegmentRecord;

pub type TempAuthTokensMap = Arc<Mutex<HashMap<String, (DateTime<Utc>, String)>>>;

pub type PendingTotpSetupState = Arc<Mutex<Option<(String, Vec<String>, DateTime<Utc>)>>>;

#[derive(Clone)]
pub struct ServerState {
    pub service: Arc<LocalAdminService>,
    pub dns_blocklist: Option<Arc<BlocklistStore>>,
    pub dns_cache: Option<Arc<famshield_dns::DnsCache>>,
    pub dns_balancer: Option<Arc<famshield_dns::NumaDnsBalancer>>,
    pub sessions: Arc<Mutex<HashMap<String, DateTime<Utc>>>>,
    pub temp_auth_tokens: TempAuthTokensMap,
    pub pending_totp_setup: PendingTotpSetupState,
    pub active_app: Arc<Mutex<String>>,
    pub is_terminating: Option<Arc<std::sync::atomic::AtomicBool>>,
}

pub struct LocalAdminServer {
    state: ServerState,
}

impl LocalAdminServer {
    pub fn new(
        service: Arc<LocalAdminService>,
        dns_blocklist: Option<Arc<BlocklistStore>>,
    ) -> Self {
        Self {
            state: ServerState {
                service,
                dns_blocklist,
                dns_cache: None,
                dns_balancer: None,
                sessions: Arc::new(Mutex::new(HashMap::new())),
                temp_auth_tokens: Arc::new(Mutex::new(HashMap::new())),
                pending_totp_setup: Arc::new(Mutex::new(None)),
                active_app: Arc::new(Mutex::new("Roblox.exe".to_string())),
                is_terminating: None,
            },
        }
    }

    pub fn with_terminating_flag(mut self, flag: Arc<std::sync::atomic::AtomicBool>) -> Self {
        self.state.is_terminating = Some(flag);
        self
    }

    pub fn with_dns_cache(mut self, dns_cache: Arc<famshield_dns::DnsCache>) -> Self {
        self.state.dns_cache = Some(dns_cache);
        self
    }

    pub fn with_dns_balancer(mut self, dns_balancer: Arc<famshield_dns::NumaDnsBalancer>) -> Self {
        self.state.dns_balancer = Some(dns_balancer);
        self
    }

    /// Pre-generate and register an active session token (e.g. for tray PIN dialog direct launch)
    pub async fn create_session_token(&self) -> String {
        let token = generate_token();
        let expires_at = Utc::now() + Duration::hours(1);
        self.state
            .sessions
            .lock()
            .await
            .insert(token.clone(), expires_at);
        token
    }

    /// Update current active foreground application dynamically
    pub async fn set_active_app(&self, app_name: &str) {
        *self.state.active_app.lock().await = app_name.to_string();

        let seg_id = format!("seg-{}", Utc::now().timestamp_millis());
        let _ = self
            .state
            .service
            .storage()
            .record_usage_segment(&UsageSegmentRecord {
                segment_id: seg_id,
                child_id: "local_child".to_string(),
                rule_id: app_name.to_string(),
                start_utc: Utc::now(),
                duration_ms: 60_000,
                state: "committed".to_string(),
            });
    }

    /// Access reference to underlying server state
    pub fn state(&self) -> &ServerState {
        &self.state
    }

    /// Construct Axum router
    pub fn router(&self) -> Router {
        Router::new()
            .route("/", get(handle_index))
            .route("/assets/{*path}", get(handle_assets))
            .route("/api/auth/login", post(handle_login))
            .route("/api/auth/login/totp", post(handle_login_totp))
            .route("/api/auth/logout", post(handle_logout))
            .route("/api/auth/verify", get(handle_verify))
            .route("/api/auth/totp/status", get(handle_get_totp_status))
            .route("/api/auth/totp/setup", post(handle_setup_totp))
            .route("/api/auth/totp/confirm", post(handle_confirm_totp))
            .route("/api/auth/totp/disable", post(handle_disable_totp))
            .route("/api/status", get(handle_status))
            .route("/api/activity/summary", get(handle_get_activity_summary))
            .route("/api/activity/apps/summary", get(handle_get_apps_summary))
            .route(
                "/api/activity/apps/categories",
                get(handle_get_app_categories),
            )
            .route("/api/activity/apps/report", get(handle_get_time_report))
            .route(
                "/api/activity/apps/sync-to-server",
                post(handle_sync_apps_to_server),
            )
            .route("/api/activity/reset", post(handle_reset_reports))
            .route(
                "/api/rules",
                get(handle_list_rules).post(handle_create_rule),
            )
            .route("/api/rules/bulk", post(handle_bulk_create_rules))
            .route("/api/rules/sync-to-cloud", post(handle_sync_rules_to_cloud))
            .route(
                "/api/rules/pull-from-cloud",
                post(handle_pull_rules_from_cloud),
            )
            .route(
                "/api/policy/cloud",
                get(handle_get_cloud_policy).post(handle_update_cloud_policy),
            )
            .route("/api/filter/pause", post(handle_filter_pause))
            .route("/api/filter/unpause", post(handle_filter_unpause))
            .route("/api/filter/inspect", post(handle_filter_inspect))
            .route("/api/rules/{rule_id}", delete(handle_delete_rule))
            .route("/api/logs/dns", get(handle_list_logs))
            .route("/api/logs/dns/summary", get(handle_get_dns_summary))
            .route("/api/logs/dns/hourly", get(handle_get_dns_hourly))
            .route("/api/logs/system", get(handle_get_system_logs))
            .route(
                "/api/logs/system/download",
                get(handle_download_system_logs),
            )
            .route(
                "/api/errors",
                get(handle_list_errors).delete(handle_clear_errors),
            )
            .route("/api/errors/push-nats", post(handle_push_errors_nats))
            .route("/api/evidence", get(handle_list_evidence))
            .route(
                "/api/evidence/{evidence_id}/thumbnail",
                get(handle_get_thumbnail),
            )
            .route("/api/settings/pin", post(handle_change_pin))
            .route(
                "/api/settings/server",
                get(handle_get_server_config).post(handle_update_server_config),
            )
            .route(
                "/api/settings/server/test",
                post(handle_test_server_connection),
            )
            .route("/api/settings/server/sync", post(handle_sync_server))
            .route(
                "/api/settings/system",
                get(handle_get_system_settings).post(handle_update_system_settings),
            )
            .route(
                "/api/settings/dns-retention",
                get(handle_get_dns_retention).post(handle_set_dns_retention),
            )
            .route(
                "/api/settings/dns-retention/prune-now",
                post(handle_prune_dns_retention_now),
            )
            .route(
                "/api/settings/capture-blur",
                get(handle_get_capture_blur).post(handle_set_capture_blur),
            )
            .route(
                "/api/settings/protection-mode",
                get(handle_get_protection_mode).post(handle_set_protection_mode),
            )
            .route(
                "/api/settings/silent-mode",
                get(handle_get_silent_mode).post(handle_set_silent_mode),
            )
            .route("/api/backup/export", get(handle_export_backup))
            .route("/api/backup/restore", post(handle_restore_backup))
            .route("/api/cloud/enroll", post(handle_cloud_enroll))
            .route("/api/cloud/disenroll", post(handle_cloud_disenroll))
            .route("/api/setup/status", get(handle_setup_status))
            .route("/api/setup/initialize", post(handle_setup_initialize))
            .route(
                "/api/keywords",
                get(handle_get_keywords).post(handle_add_keyword),
            )
            .route("/api/keywords/{keyword_id}", delete(handle_delete_keyword))
            .route("/api/keywords/toggle", post(handle_toggle_keyword))
            .route(
                "/api/keywords/monitoring",
                get(handle_get_keyboard_monitoring).post(handle_toggle_keyboard_monitoring),
            )
            .route(
                "/api/keywords/violation",
                post(handle_record_keyword_violation),
            )
            .route(
                "/api/system/session-helper/start",
                post(handle_start_session_helper),
            )
            .route("/api/system/uninstall", post(handle_system_uninstall))
            .route("/api/system/service/restart", post(handle_restart_service))
            .route("/api/system/update/check", get(handle_check_update))
            .route(
                "/api/system/update/url",
                get(handle_get_update_urls).post(handle_update_urls),
            )
            .route(
                "/api/dns/config",
                get(handle_get_dns_config).post(handle_set_dns_config),
            )
            .route("/api/dns/cache/flush", post(handle_flush_dns_cache))
            .route("/api/system/info", get(handle_get_system_info))
            .fallback(handle_fallback)
            .with_state(self.state.clone())
    }

    /// Launch HTTP server on specified address (e.g., "127.0.0.1:38123"), automatically falling back if port is reserved by Hyper-V/OS
    pub async fn run(
        &self,
        bind_addr: &str,
        mut shutdown_rx: broadcast::Receiver<()>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let app = self.router();

        let listener = match tokio::net::TcpListener::bind(bind_addr).await {
            Ok(l) => {
                if let Ok(addr) = l.local_addr() {
                    let port_file = std::env::var("ProgramData")
                        .map(std::path::PathBuf::from)
                        .unwrap_or_else(|_| std::path::PathBuf::from("."))
                        .join("FamShield")
                        .join("admin_port.txt");

                    let _ = std::fs::write(port_file, addr.port().to_string());
                }
                l
            }
            Err(e) => {
                warn!(
                    "Failed to bind Local Admin server to {} ({}). Attempting fallback ports...",
                    bind_addr, e
                );

                let fallback_ports = [38123, 18123, 28123, 48123];
                let mut bound = None;

                for &port in &fallback_ports {
                    let fallback_addr = format!("127.0.0.1:{}", port);
                    if let Ok(l) = tokio::net::TcpListener::bind(&fallback_addr).await {
                        info!(
                            "Successfully bound Local Admin server to fallback {}",
                            fallback_addr
                        );

                        let port_file = std::env::var("ProgramData")
                            .map(std::path::PathBuf::from)
                            .unwrap_or_else(|_| std::path::PathBuf::from("."))
                            .join("FamShield")
                            .join("admin_port.txt");

                        let _ = std::fs::write(port_file, port.to_string());
                        bound = Some(l);
                        break;
                    }
                }

                match bound {
                    Some(l) => l,
                    None => return Err(Box::new(e)),
                }
            }
        };

        let local_addr = listener.local_addr()?;

        info!(
            "FamShield Local Admin Web Server listening on http://{}",
            local_addr
        );

        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                let _ = shutdown_rx.recv().await;
                info!("Local Admin Web Server shutting down gracefully");
            })
            .await?;

        Ok(())
    }
}

// ================= Helpers =================

pub(crate) fn generate_token() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

pub(crate) fn extract_token(headers: &HeaderMap, query_token: Option<&str>) -> Option<String> {
    if let Some(tok) = query_token {
        let trimmed = tok.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    if let Some(auth) = headers
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
    {
        if let Some(token) = auth.strip_prefix("Bearer ") {
            return Some(token.trim().to_string());
        }
    }

    if let Some(custom) = headers
        .get("X-FamShield-Token")
        .and_then(|h| h.to_str().ok())
    {
        return Some(custom.trim().to_string());
    }

    if let Some(cookie) = headers.get(header::COOKIE).and_then(|h| h.to_str().ok()) {
        for part in cookie.split(';') {
            let part = part.trim();
            if let Some(val) = part.strip_prefix("famshield_session=") {
                return Some(val.to_string());
            }
        }
    }

    None
}

pub(crate) async fn check_auth(
    state: &ServerState,
    headers: &HeaderMap,
    query_token: Option<&str>,
) -> bool {
    if let Some(token) = extract_token(headers, query_token) {
        let mut sessions = state.sessions.lock().await;

        if let Some(expires_at) = sessions.get(&token) {
            if *expires_at > Utc::now() {
                // Slide session expiration window (60 minutes)
                sessions.insert(token, Utc::now() + Duration::minutes(60));
                return true;
            }
        }
    }

    false
}

const DEFAULT_ADULT_SEEDS: &[&str] = &[
    "pornhub.com",
    "xvideos.com",
    "xnxx.com",
    "redtube.com",
    "onlyfans.com",
    "chaturbate.com",
    "stripchat.com",
    "livejasmin.com",
    "spankbang.com",
];

const DEFAULT_GAMBLING_SEEDS: &[&str] = &[
    "bet365.com",
    "1xbet.com",
    "w88.com",
    "fun88.com",
    "m88.com",
    "fb88.com",
    "dafabet.com",
    "pokerstars.com",
    "stake.com",
];

const DEFAULT_VIOLENCE_SEEDS: &[&str] = &[
    "bestgore.com",
    "crazyshit.com",
    "liveleak.com",
    "kaotic.com",
    "theync.com",
];

const DEFAULT_SOCIAL_SEEDS: &[&str] = &[
    "tiktok.com",
    "facebook.com",
    "instagram.com",
    "twitter.com",
    "x.com",
    "reddit.com",
    "snapchat.com",
];

const DEFAULT_GAMING_SEEDS: &[&str] = &[
    "roblox.com",
    "steampowered.com",
    "steamcommunity.com",
    "riotgames.com",
    "epicgames.com",
    "garena.vn",
];

pub(crate) fn sync_dns_blocklist(state: &ServerState) {
    if let Some(blocklist) = &state.dns_blocklist {
        blocklist.set_monitor_mode(state.service.is_monitor_mode());

        let mut general_domains: Vec<String> = Vec::new();
        let mut custom_rules: Vec<String> = Vec::new();
        let mut allowed_domains: Vec<String> = Vec::new();

        // 1. Load from cached_blocklist if present (atomic pre-compiled list from cloud sync/NATS)
        if let Ok(Some(cached_json)) = state.service.storage().get_meta("cached_blocklist") {
            if let Ok(domains) = serde_json::from_str::<Vec<String>>(&cached_json) {
                general_domains.extend(domains);
            }
        }

        // 2. Load from local rules in SQLite (Highest custom priority on client!)
        if let Ok(rules) = state.service.list_rules() {
            for r in rules {
                if (r.rule_type == "domain" || r.rule_type == "pattern") && r.enabled {
                    let is_allowed =
                        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&r.payload) {
                            val.get("action")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_uppercase())
                                == Some("ALLOW".to_string())
                        } else {
                            false
                        };
                    if is_allowed {
                        allowed_domains.push(r.rule_id);
                    } else {
                        custom_rules.push(r.rule_id);
                    }
                }
            }
        }

        // 3. Merge from cached cloud policy (if present)
        if let Ok(Some(cached)) = state.service.storage().get_meta("cached_cloud_policy") {
            if let Ok(cloud_policy) = serde_json::from_str::<serde_json::Value>(&cached) {
                let block_adult = cloud_policy
                    .get("block_adult")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let block_gambling = cloud_policy
                    .get("block_gambling")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let block_violence = cloud_policy
                    .get("block_violence")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let block_social = cloud_policy
                    .get("block_social")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let block_gaming = cloud_policy
                    .get("block_gaming")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                // Include standard system seeds when category is blocked
                if block_adult {
                    for d in DEFAULT_ADULT_SEEDS {
                        general_domains.push(d.to_string());
                    }
                }
                if block_gambling {
                    for d in DEFAULT_GAMBLING_SEEDS {
                        general_domains.push(d.to_string());
                    }
                }
                if block_violence {
                    for d in DEFAULT_VIOLENCE_SEEDS {
                        general_domains.push(d.to_string());
                    }
                }
                if block_social {
                    for d in DEFAULT_SOCIAL_SEEDS {
                        general_domains.push(d.to_string());
                    }
                }
                if block_gaming {
                    for d in DEFAULT_GAMING_SEEDS {
                        general_domains.push(d.to_string());
                    }
                }

                // Category custom domains
                if let Some(cat_map) = cloud_policy
                    .get("category_custom_domains")
                    .and_then(|v| v.as_object())
                {
                    for (cat, arr) in cat_map {
                        let cat_lower = cat.to_lowercase();
                        let norm_cat = match cat_lower.trim() {
                            "adult" | "18+" | "18" | "porn" | "xxx" | "nsfw" => "adult",
                            "gambling" | "bet" | "casino" => "gambling",
                            "violence" | "gore" => "violence",
                            "social" => "social",
                            "gaming" | "game" => "gaming",
                            other => other,
                        };
                        let is_blocked = match norm_cat {
                            "adult" => block_adult,
                            "gambling" => block_gambling,
                            "violence" => block_violence,
                            "social" => block_social,
                            "gaming" => block_gaming,
                            _ => false,
                        };
                        if is_blocked {
                            if let Some(arr) = arr.as_array() {
                                for d in arr {
                                    if let Some(s) = d.as_str() {
                                        general_domains.push(s.to_string());
                                    }
                                }
                            }
                        }
                    }
                }

                if let Some(domains) = cloud_policy
                    .get("custom_blocked_domains")
                    .and_then(|v| v.as_array())
                {
                    for d_val in domains {
                        if let Some(d) = d_val.as_str() {
                            custom_rules.push(d.to_string());
                        }
                    }
                }

                if let Some(domains) = cloud_policy
                    .get("custom_allowed_domains")
                    .and_then(|v| v.as_array())
                {
                    for d_val in domains {
                        if let Some(d) = d_val.as_str() {
                            allowed_domains.push(d.to_string());
                        }
                    }
                }
            }
        }

        // Apply to BlocklistStore with exact hierarchical priority:
        // Priority 1: Whitelist (allowed_domains)
        // Priority 2: Custom Client Rules (domains + name patterns like "thiendia")
        // Priority 3: General Threat Feed
        blocklist.replace_whitelist(allowed_domains);
        blocklist.replace_custom_blocked(custom_rules);
        blocklist.replace_blocklist(general_domains);
    }

    if let Some(cache) = &state.dns_cache {
        cache.flush();
    }
}

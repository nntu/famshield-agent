use super::LocalAdminService;
use crate::service::types::{
    LocalAdminError, ServerConfig, SystemSettings, UpdateServerConfigRequest,
    UpdateSystemSettingsRequest,
};
use chrono::Utc;

impl LocalAdminService {
    pub fn get_server_config(&self) -> Result<ServerConfig, LocalAdminError> {
        let enrolled_id = self
            .storage
            .get_meta("enrolled_device_id")?
            .unwrap_or_default();
        let is_enrolled = !enrolled_id.trim().is_empty();

        let server_url = self
            .storage
            .get_meta("server_url")?
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "https://famshield.ttcantho.io.vn".to_string());

        let tenant_id = self
            .storage
            .get_meta("enrolled_tenant_id")?
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "tenant-default".to_string());

        let custom_device_name = self.storage.get_meta("device_name")?;
        let device_name = famshield_windows::get_disambiguated_device_name(
            custom_device_name.as_deref().filter(|s| !s.is_empty()),
        );

        let enrollment_code = self
            .storage
            .get_meta("enrollment_code")?
            .filter(|s| !s.is_empty());

        let sync_interval_secs = self
            .storage
            .get_meta("sync_interval_secs")?
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(60);

        let tls_verify = self
            .storage
            .get_meta("tls_verify")?
            .map(|s| s != "false")
            .unwrap_or(true);

        let last_sync_time = self
            .storage
            .get_meta("last_sync_time")?
            .filter(|s| !s.is_empty());

        let policy_version = self
            .storage
            .get_meta("policy_version")?
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(1);

        let version_check_url = self
            .storage
            .get_meta("version_check_url")?
            .filter(|s| !s.is_empty());

        let nats_status = self
            .storage
            .get_meta("nats_status")?
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "disconnected".to_string());

        let (operating_mode, connection_status) = if is_enrolled {
            let conn = if nats_status == "connected" {
                "Connected (Real-time NATS)".to_string()
            } else {
                "Reconnecting (HTTP Fallback)".to_string()
            };
            ("OnlineManaged".to_string(), conn)
        } else {
            ("StandaloneFree".to_string(), "Disconnected".to_string())
        };

        Ok(ServerConfig {
            operating_mode,
            server_url,
            version_check_url,
            tenant_id,
            device_id: if is_enrolled {
                enrolled_id
            } else {
                "standalone-local-001".to_string()
            },
            device_name,
            enrollment_code,
            sync_interval_secs,
            tls_verify,
            connection_status,
            nats_status,
            last_sync_time,
            policy_version,
        })
    }

    /// Update centralized server settings or initiate enrollment
    pub fn update_server_config(
        &self,
        req: UpdateServerConfigRequest,
    ) -> Result<ServerConfig, LocalAdminError> {
        if let Some(url) = &req.server_url {
            let trimmed = url.trim();
            if !trimmed.starts_with("http://") && !trimmed.starts_with("https://") {
                return Err(LocalAdminError::PolicyInvalid(
                    "Server URL must start with http:// or https://".to_string(),
                ));
            }
            self.storage.set_meta("server_url", trimmed)?;
            // Clean cached release on server change
            let _ = self.storage.delete_meta("available_release_json");
            let _ = self.storage.delete_meta("available_release_ver");
        }

        if let Some(v_url) = &req.version_check_url {
            let trimmed = v_url.trim();
            if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("auto") {
                let _ = self.storage.delete_meta("version_check_url");
            } else {
                if !trimmed.starts_with("http://") && !trimmed.starts_with("https://") {
                    return Err(LocalAdminError::PolicyInvalid(
                        "Version check URL must start with http:// or https://".to_string(),
                    ));
                }
                self.storage.set_meta("version_check_url", trimmed)?;
            }
            let _ = self.storage.delete_meta("available_release_json");
            let _ = self.storage.delete_meta("available_release_ver");
        }

        if let Some(tenant) = &req.tenant_id {
            self.storage.set_meta("enrolled_tenant_id", tenant.trim())?;
        }

        if let Some(name) = &req.device_name {
            self.storage.set_meta("device_name", name.trim())?;
        }

        if let Some(interval) = req.sync_interval_secs {
            let clamped = interval.clamp(10, 86400);
            self.storage
                .set_meta("sync_interval_secs", &clamped.to_string())?;
        }

        if let Some(tls) = req.tls_verify {
            self.storage
                .set_meta("tls_verify", if tls { "true" } else { "false" })?;
        }

        if let Some(code) = &req.enrollment_code {
            let trimmed = code.trim();
            if !trimmed.is_empty() {
                self.storage.set_meta("enrollment_code", trimmed)?;
                let dev_id = format!("dev-{}", trimmed);
                self.storage.set_meta("enrolled_device_id", &dev_id)?;
                self.storage
                    .set_meta("last_sync_time", &Utc::now().to_rfc3339())?;
            }
        }

        if let Some(mode) = &req.operating_mode {
            if mode == "StandaloneFree" {
                self.storage.set_meta("enrolled_device_id", "")?;
                self.storage.set_meta("enrollment_code", "")?;
            }
        }

        self.get_server_config()
    }

    /// Force immediate sync timestamp update
    pub fn trigger_sync(&self) -> Result<String, LocalAdminError> {
        let now_str = Utc::now().to_rfc3339();
        self.storage.set_meta("last_sync_time", &now_str)?;
        let cur_ver = self
            .storage
            .get_meta("policy_version")?
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(1);
        self.storage
            .set_meta("policy_version", &(cur_ver + 1).to_string())?;
        Ok(now_str)
    }

    /// Check whether the agent is enrolled with a centralized server
    pub fn is_enrolled(&self) -> bool {
        self.storage
            .get_meta("enrolled_device_id")
            .ok()
            .flatten()
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false)
    }

    /// Check whether monitor mode is active (observe & log only, do not block).
    pub fn is_monitor_mode(&self) -> bool {
        self.storage
            .get_meta("monitor_mode")
            .ok()
            .flatten()
            .map(|s| s == "true" || s == "1")
            .unwrap_or(false)
    }

    /// Enable or disable monitor mode.
    pub fn set_monitor_mode(&self, enabled: bool) -> Result<(), LocalAdminError> {
        self.storage
            .set_meta("monitor_mode", if enabled { "true" } else { "false" })?;
        Ok(())
    }

    /// Check whether silent monitoring mode is active (suppress all toast popups).
    pub fn is_silent_mode(&self) -> bool {
        self.storage
            .get_meta("silent_mode")
            .ok()
            .flatten()
            .map(|s| s == "true" || s == "1")
            .unwrap_or(false)
    }

    /// Enable or disable silent monitoring mode.
    pub fn set_silent_mode(&self, enabled: bool) -> Result<(), LocalAdminError> {
        self.storage
            .set_meta("silent_mode", if enabled { "true" } else { "false" })?;
        Ok(())
    }

    /// Retrieve configured DNS blocking action (e.g. "nxdomain", "refused", "zero_ip").
    pub fn get_dns_block_action(&self) -> String {
        self.storage
            .get_meta("dns_block_action")
            .ok()
            .flatten()
            .unwrap_or_else(|| "nxdomain".to_string())
    }

    /// Set DNS blocking action.
    pub fn set_dns_block_action(&self, action: &str) -> Result<(), LocalAdminError> {
        self.storage.set_meta("dns_block_action", action)?;
        Ok(())
    }

    pub fn list_all_errors(
        &self,
        limit: usize,
    ) -> Result<Vec<famshield_storage::ErrorLogRecord>, LocalAdminError> {
        Ok(self.storage.list_all_errors(limit)?)
    }

    pub fn get_system_settings(&self) -> Result<SystemSettings, LocalAdminError> {
        let server_url = self
            .storage
            .get_meta("server_url")?
            .unwrap_or_else(|| "https://famshield.ttcantho.io.vn".to_string());

        let version_check_url = self
            .storage
            .get_meta("version_check_url")?
            .unwrap_or_default();

        let nats_url = self.storage.get_meta("nats_url")?.unwrap_or_default();

        let sync_interval_secs = self
            .storage
            .get_meta("sync_interval_secs")?
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(15);

        let heartbeat_interval_secs = self
            .storage
            .get_meta("heartbeat_interval_secs")?
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(60);

        let dns_upstream = self
            .storage
            .get_meta("dns_upstream")?
            .unwrap_or_else(|| "1.1.1.1".to_string());

        let dns_block_action = self
            .storage
            .get_meta("dns_block_action")?
            .unwrap_or_else(|| "nxdomain".to_string());

        let dns_cache_ttl_secs = self
            .storage
            .get_meta("dns_cache_ttl_secs")?
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(300);

        let dns_timeout_ms = self
            .storage
            .get_meta("dns_timeout_ms")?
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(2000);

        let silent_mode = self.is_silent_mode();

        let monitor_mode = self.is_monitor_mode();

        let screenshot_blur_mode = self
            .storage
            .get_meta("screenshot_blur_mode")?
            .unwrap_or_else(|| "full".to_string());

        let dns_log_retention_days = self
            .storage
            .get_meta("dns_log_retention_days")?
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(7);

        let dns_auto_prune_enabled = self
            .storage
            .get_meta("dns_auto_prune_enabled")?
            .map(|s| s == "true" || s == "1")
            .unwrap_or(true);

        let keyboard_monitoring_enabled = self.is_keyboard_monitoring_enabled();

        let max_pin_attempts = self
            .storage
            .get_meta("max_pin_attempts")?
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(5);

        let pin_lockout_duration_mins = self
            .storage
            .get_meta("pin_lockout_duration_mins")?
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(5);

        Ok(SystemSettings {
            server_url,

            version_check_url,

            nats_url,

            sync_interval_secs,

            heartbeat_interval_secs,

            dns_upstream,

            dns_block_action,

            dns_cache_ttl_secs,

            dns_timeout_ms,

            silent_mode,

            monitor_mode,

            screenshot_blur_mode,

            dns_log_retention_days,

            dns_auto_prune_enabled,

            keyboard_monitoring_enabled,

            max_pin_attempts,

            pin_lockout_duration_mins,
        })
    }

    pub fn update_system_settings(
        &self,

        req: UpdateSystemSettingsRequest,
    ) -> Result<SystemSettings, LocalAdminError> {
        if let Some(url) = req.server_url {
            let trimmed = url.trim();

            if !trimmed.starts_with("http://") && !trimmed.starts_with("https://") {
                return Err(LocalAdminError::PolicyInvalid(
                    "Server URL must start with http:// or https://".to_string(),
                ));
            }

            self.storage.set_meta("server_url", trimmed)?;

            let _ = self.storage.delete_meta("available_release_json");

            let _ = self.storage.delete_meta("available_release_ver");
        }

        if let Some(v_url) = req.version_check_url {
            let trimmed = v_url.trim();

            if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("auto") {
                let _ = self.storage.delete_meta("version_check_url");
            } else {
                if !trimmed.starts_with("http://") && !trimmed.starts_with("https://") {
                    return Err(LocalAdminError::PolicyInvalid(
                        "Version check URL must start with http:// or https://".to_string(),
                    ));
                }

                self.storage.set_meta("version_check_url", trimmed)?;
            }

            let _ = self.storage.delete_meta("available_release_json");

            let _ = self.storage.delete_meta("available_release_ver");
        }

        if let Some(nats) = req.nats_url {
            let trimmed = nats.trim();

            if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("auto") {
                let _ = self.storage.delete_meta("nats_url");
            } else {
                self.storage.set_meta("nats_url", trimmed)?;
            }
        }

        if let Some(sync_sec) = req.sync_interval_secs {
            let clamped = sync_sec.clamp(5, 3600);

            self.storage
                .set_meta("sync_interval_secs", &clamped.to_string())?;
        }

        if let Some(hb_sec) = req.heartbeat_interval_secs {
            let clamped = hb_sec.clamp(10, 86400);

            self.storage
                .set_meta("heartbeat_interval_secs", &clamped.to_string())?;
        }

        if let Some(dns) = req.dns_upstream {
            let trimmed = dns.trim();

            if !trimmed.is_empty() {
                self.storage.set_meta("dns_upstream", trimmed)?;
            }
        }

        if let Some(action) = req.dns_block_action {
            let _ = self.set_dns_block_action(&action);
        }

        if let Some(ttl) = req.dns_cache_ttl_secs {
            let clamped = ttl.clamp(10, 86400);

            self.storage
                .set_meta("dns_cache_ttl_secs", &clamped.to_string())?;
        }

        if let Some(timeout) = req.dns_timeout_ms {
            let clamped = timeout.clamp(200, 10000);

            self.storage
                .set_meta("dns_timeout_ms", &clamped.to_string())?;
        }

        if let Some(silent) = req.silent_mode {
            let _ = self.set_silent_mode(silent);
        }

        if let Some(monitor) = req.monitor_mode {
            let _ = self.set_monitor_mode(monitor);
        }

        if let Some(blur) = req.screenshot_blur_mode {
            let b = blur.trim().to_lowercase();

            if b == "full" || b == "reduced" || b == "none" {
                self.storage.set_meta("screenshot_blur_mode", &b)?;
            }
        }

        if let Some(days) = req.dns_log_retention_days {
            let clamped = days.clamp(1, 365);

            self.storage
                .set_meta("dns_log_retention_days", &clamped.to_string())?;
        }

        if let Some(auto_prune) = req.dns_auto_prune_enabled {
            self.storage.set_meta(
                "dns_auto_prune_enabled",
                if auto_prune { "true" } else { "false" },
            )?;
        }

        if let Some(kb) = req.keyboard_monitoring_enabled {
            let _ = self.set_keyboard_monitoring_enabled(kb);
        }

        if let Some(attempts) = req.max_pin_attempts {
            let clamped = attempts.clamp(3, 20);

            self.storage
                .set_meta("max_pin_attempts", &clamped.to_string())?;
        }

        if let Some(duration) = req.pin_lockout_duration_mins {
            let clamped = duration.clamp(1, 1440);

            self.storage
                .set_meta("pin_lockout_duration_mins", &clamped.to_string())?;
        }

        self.get_system_settings()
    }

    pub fn clear_errors(&self) -> Result<usize, LocalAdminError> {
        Ok(self.storage.clear_errors()?)
    }
}

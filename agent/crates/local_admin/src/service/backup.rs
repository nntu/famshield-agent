use super::LocalAdminService;
use crate::service::types::{BackupPayload, BackupSettings, LocalAdminError, RestoreResult};
use chrono::Utc;

impl LocalAdminService {
    /// Create complete backup snapshot payload with SHA-256 integrity checksum.
    pub fn create_backup(&self) -> Result<BackupPayload, LocalAdminError> {
        let now = Utc::now().to_rfc3339();
        let rules = self.storage.list_local_rules()?;
        let server_cfg = self.get_server_config()?;
        let monitor_mode = self.is_monitor_mode();
        let silent_mode = self.is_silent_mode();

        let settings = BackupSettings {
            operating_mode: server_cfg.operating_mode,
            server_url: server_cfg.server_url,
            tenant_id: server_cfg.tenant_id,
            device_name: server_cfg.device_name,
            sync_interval_secs: server_cfg.sync_interval_secs,
            tls_verify: server_cfg.tls_verify,
            monitor_mode,
            silent_mode,
        };

        let settings_json = serde_json::to_string(&settings)
            .map_err(|e| LocalAdminError::Backup(format!("Serialize settings error: {}", e)))?;
        let rules_json = serde_json::to_string(&rules)
            .map_err(|e| LocalAdminError::Backup(format!("Serialize rules error: {}", e)))?;

        let format_version = "1.0".to_string();
        let app_version = env!("CARGO_PKG_VERSION").to_string();
        let device_id = server_cfg.device_id;

        let checksum = BackupPayload::compute_checksum(
            &format_version,
            &now,
            &device_id,
            &settings_json,
            &rules_json,
        );

        Ok(BackupPayload {
            format_version,
            created_at: now,
            app_version,
            device_id,
            settings,
            rules,
            checksum,
        })
    }

    /// Restore configuration and rules from a backup payload after validating checksum.
    pub fn restore_backup(&self, payload: BackupPayload) -> Result<RestoreResult, LocalAdminError> {
        if payload.format_version != "1.0" {
            return Err(LocalAdminError::Backup(format!(
                "Phiên bản định dạng sao lưu không được hỗ trợ: {}",
                payload.format_version
            )));
        }

        if !payload.verify_checksum() {
            return Err(LocalAdminError::Backup(
                "Tập tin sao lưu bị hỏng hoặc đã bị chỉnh sửa trái phép (Sai lệch mã checksum SHA-256)".to_string(),
            ));
        }

        self.set_monitor_mode(payload.settings.monitor_mode)?;
        let _ = self.set_silent_mode(payload.settings.silent_mode);
        self.storage
            .set_meta("operating_mode", &payload.settings.operating_mode)?;
        self.storage
            .set_meta("server_url", &payload.settings.server_url)?;
        self.storage
            .set_meta("enrolled_tenant_id", &payload.settings.tenant_id)?;
        self.storage
            .set_meta("device_name", &payload.settings.device_name)?;
        self.storage.set_meta(
            "sync_interval_secs",
            &payload.settings.sync_interval_secs.to_string(),
        )?;
        self.storage.set_meta(
            "tls_verify",
            if payload.settings.tls_verify {
                "true"
            } else {
                "false"
            },
        )?;

        let mut restored_count = 0;
        for rule in &payload.rules {
            self.storage.upsert_local_rule(rule)?;
            restored_count += 1;
        }

        let now = Utc::now().to_rfc3339();
        self.storage.set_meta("last_restore_time", &now)?;

        Ok(RestoreResult {
            success: true,
            rules_restored: restored_count,
            message: format!(
                "Khôi phục thành công {} quy tắc và toàn bộ cấu hình hệ thống.",
                restored_count
            ),
        })
    }
}

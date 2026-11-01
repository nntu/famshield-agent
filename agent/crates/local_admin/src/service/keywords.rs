use super::LocalAdminService;
use crate::service::types::{
    AddKeywordRequest, KeywordConfigResponse, KeywordRuleDto, KeywordViolationDto, LocalAdminError,
};
use chrono::Utc;
use famshield_storage::CustomKeywordRecord;
use sha2::Digest;

impl LocalAdminService {
    pub fn is_keyboard_monitoring_enabled(&self) -> bool {
        self.storage
            .get_meta("keyboard_monitoring_enabled")
            .unwrap_or(None)
            .map(|s| s != "false")
            .unwrap_or(true)
    }

    pub fn set_keyboard_monitoring_enabled(&self, enabled: bool) -> Result<(), LocalAdminError> {
        self.storage.set_meta(
            "keyboard_monitoring_enabled",
            if enabled { "true" } else { "false" },
        )?;
        Ok(())
    }

    pub fn get_keyword_config(&self) -> Result<KeywordConfigResponse, LocalAdminError> {
        let keyboard_monitoring_enabled = self.is_keyboard_monitoring_enabled();
        let violations_today_count = self.storage.count_keyword_violations_today().unwrap_or(0);

        let mut rules = Vec::new();
        // 1. Default system rules
        for r in famshield_core::keyword::default_keyword_rules() {
            let override_key = format!("kw_disabled_{}", r.id);
            let is_disabled = self.storage.get_meta(&override_key)?.as_deref() == Some("1");
            rules.push(KeywordRuleDto {
                id: r.id,
                pattern: r.pattern,
                category: format!("{:?}", r.category),
                category_name: r.category.display_name().to_string(),
                severity: r.severity.as_str().to_string(),
                enabled: !is_disabled,
                is_custom: false,
            });
        }

        // 2. Custom rules from storage
        let custom_list = self.storage.list_custom_keywords().unwrap_or_default();
        for c in custom_list {
            rules.push(KeywordRuleDto {
                id: c.id,
                pattern: c.pattern,
                category: c.category.clone(),
                category_name: if c.category == "Custom" {
                    "Tùy chỉnh phụ huynh".to_string()
                } else {
                    c.category
                },
                severity: c.severity,
                enabled: c.enabled,
                is_custom: true,
            });
        }

        // 3. Recent violations
        let raw_violations = self.storage.list_keyword_violations(30).unwrap_or_default();
        let recent_violations = raw_violations
            .into_iter()
            .map(|v| KeywordViolationDto {
                id: v.id,
                timestamp: v.timestamp_utc,
                app_name: v.app_name,
                keyword_matched: v.keyword_matched,
                category: v.category,
                severity: v.severity,
                snippet_masked: v.snippet_masked,
            })
            .collect();

        let total_rules_count = rules.len();

        Ok(KeywordConfigResponse {
            keyboard_monitoring_enabled,
            total_rules_count,
            violations_today_count,
            rules,
            recent_violations,
        })
    }

    pub fn add_custom_keyword(
        &self,
        req: AddKeywordRequest,
    ) -> Result<KeywordRuleDto, LocalAdminError> {
        let trimmed = req.pattern.trim();
        if trimmed.is_empty() {
            return Err(LocalAdminError::PolicyInvalid(
                "Từ khóa không được để trống".into(),
            ));
        }

        let hash_str = format!("{:x}", sha2::Sha256::digest(trimmed.as_bytes()));
        let id = format!("custom_kw_{}", &hash_str[0..12]);
        let cat = req
            .category
            .unwrap_or_else(|| "Tùy chỉnh phụ huynh".to_string());
        let sev = req.severity.unwrap_or_else(|| "medium".to_string());

        let record = CustomKeywordRecord {
            id: id.clone(),
            pattern: trimmed.to_string(),
            category: cat.clone(),
            severity: sev.clone(),
            enabled: true,
            created_at: Utc::now(),
        };

        self.storage.upsert_custom_keyword(&record)?;

        Ok(KeywordRuleDto {
            id,
            pattern: trimmed.to_string(),
            category: cat.clone(),
            category_name: cat,
            severity: sev,
            enabled: true,
            is_custom: true,
        })
    }

    pub fn delete_custom_keyword(&self, id: &str) -> Result<bool, LocalAdminError> {
        let deleted = self.storage.delete_custom_keyword(id)?;
        Ok(deleted)
    }

    pub fn toggle_keyword(&self, id: &str, enabled: bool) -> Result<bool, LocalAdminError> {
        if id.starts_with("custom_kw_") {
            self.storage.toggle_custom_keyword(id, enabled)?;
            Ok(true)
        } else {
            let override_key = format!("kw_disabled_{}", id);
            self.storage
                .set_meta(&override_key, if enabled { "0" } else { "1" })?;
            Ok(true)
        }
    }
}

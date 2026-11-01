use super::StorageEngine;
use crate::engine::records::{AppCategoryEntity, AppCategoryRuleEntity};
use crate::error::StorageError;
use rusqlite::params;

impl StorageEngine {
    // App Categories and Classification Rules
    pub fn list_app_categories(&self) -> Result<Vec<AppCategoryEntity>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name_vi, name_en, icon, is_system, description_vi, description_en, sort_order
             FROM app_categories ORDER BY sort_order ASC, id ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            let is_sys_int: i32 = row.get(4)?;
            Ok(AppCategoryEntity {
                id: row.get(0)?,
                name_vi: row.get(1)?,
                name_en: row.get(2)?,
                icon: row.get(3)?,
                is_system: is_sys_int == 1,
                description_vi: row.get(5)?,
                description_en: row.get(6)?,
                sort_order: row.get(7)?,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn list_app_category_rules(&self) -> Result<Vec<AppCategoryRuleEntity>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT rule_id, category_id, pattern, match_type, app_title, is_system, priority
             FROM app_category_rules ORDER BY priority DESC, rule_id ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            let is_sys_int: i32 = row.get(5)?;
            Ok(AppCategoryRuleEntity {
                rule_id: row.get(0)?,
                category_id: row.get(1)?,
                pattern: row.get(2)?,
                match_type: row.get(3)?,
                app_title: row.get(4)?,
                is_system: is_sys_int == 1,
                priority: row.get(6)?,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn upsert_app_category(&self, cat: &AppCategoryEntity) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO app_categories (id, name_vi, name_en, icon, is_system, description_vi, description_en, sort_order)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(id) DO UPDATE SET
                name_vi = excluded.name_vi,
                name_en = excluded.name_en,
                icon = excluded.icon,
                is_system = excluded.is_system,
                description_vi = excluded.description_vi,
                description_en = excluded.description_en,
                sort_order = excluded.sort_order",
            params![
                cat.id,
                cat.name_vi,
                cat.name_en,
                cat.icon,
                if cat.is_system { 1 } else { 0 },
                cat.description_vi,
                cat.description_en,
                cat.sort_order,
            ],
        )?;
        Ok(())
    }

    pub fn upsert_app_category_rule(&self, rule: &AppCategoryRuleEntity) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO app_category_rules (rule_id, category_id, pattern, match_type, app_title, is_system, priority)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(rule_id) DO UPDATE SET
                category_id = excluded.category_id,
                pattern = excluded.pattern,
                match_type = excluded.match_type,
                app_title = excluded.app_title,
                is_system = excluded.is_system,
                priority = excluded.priority",
            params![
                rule.rule_id,
                rule.category_id,
                rule.pattern,
                rule.match_type,
                rule.app_title,
                if rule.is_system { 1 } else { 0 },
                rule.priority,
            ],
        )?;
        Ok(())
    }

    pub fn classify_app_by_rules(
        &self,
        app_name: &str,
    ) -> Result<Option<AppCategoryRuleEntity>, StorageError> {
        let rules = self.list_app_category_rules()?;
        let lower = app_name.trim().to_ascii_lowercase();
        let base = lower.trim_end_matches(".exe");

        for rule in rules {
            let pat = rule.pattern.to_ascii_lowercase();
            let matched = match rule.match_type.as_str() {
                "exact" => base == pat || lower == pat,
                "starts_with" => base.starts_with(&pat) || lower.starts_with(&pat),
                "ends_with" => base.ends_with(&pat) || lower.ends_with(&pat),
                _ => base.contains(&pat) || lower.contains(&pat),
            };
            if matched {
                return Ok(Some(rule));
            }
        }
        Ok(None)
    }
}


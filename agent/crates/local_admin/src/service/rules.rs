use super::LocalAdminService;
use crate::service::types::{CreateRuleRequest, LocalAdminError, RulePayload};
use famshield_storage::LocalRuleRecord;
use chrono::Utc;

impl LocalAdminService {
    /// Save a local rule in SQLite after verifying structure.
    pub fn create_rule(&self, req: CreateRuleRequest) -> Result<(), LocalAdminError> {
        let now = Utc::now();
        let record = LocalRuleRecord {
            rule_id: req.rule_id,
            rule_type: req.rule_type,
            payload: req.payload_json,
            enabled: req.enabled,
            created_at: now,
            updated_at: now,
            source: "local".to_string(),
        };

        self.storage.upsert_local_rule(&record)?;
        Ok(())
    }

    /// Save multiple local rules in a batch.
    pub fn create_rules_bulk(
        &self,
        requests: Vec<CreateRuleRequest>,
    ) -> Result<usize, LocalAdminError> {
        let now = Utc::now();
        let mut count = 0;
        for req in requests {
            let record = LocalRuleRecord {
                rule_id: req.rule_id,
                rule_type: req.rule_type,
                payload: req.payload_json,
                enabled: req.enabled,
                created_at: now,
                updated_at: now,
                source: "local".to_string(),
            };
            self.storage.upsert_local_rule(&record)?;
            count += 1;
        }
        Ok(count)
    }

    /// List all local rules currently stored in the standalone database.
    pub fn list_rules(&self) -> Result<Vec<LocalRuleRecord>, LocalAdminError> {
        let rules = self.storage.list_local_rules()?;
        Ok(rules)
    }

    /// Delete a local rule by ID.
    pub fn delete_rule(&self, rule_id: &str) -> Result<bool, LocalAdminError> {
        let res = self.storage.delete_local_rule(rule_id)?;
        Ok(res)
    }

}

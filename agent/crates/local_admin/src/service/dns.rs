use super::LocalAdminService;
use crate::service::types::LocalAdminError;
use chrono::{DateTime, Utc};
use famshield_storage::{
    DomainQuerySummary, HourlyFrequency, QueryLogFilter, QueryLogRecord, VisualEvidenceRecord,
};

impl LocalAdminService {
    pub fn list_query_logs(&self, limit: usize) -> Result<Vec<QueryLogRecord>, LocalAdminError> {
        let logs = self.storage.list_recent_query_logs(limit)?;
        Ok(logs)
    }

    /// Query aggregated DNS summaries grouped by domain.
    pub fn get_domain_query_summaries(
        &self,
        limit: usize,
    ) -> Result<Vec<DomainQuerySummary>, LocalAdminError> {
        let summaries = self.storage.get_domain_query_summaries(limit)?;
        Ok(summaries)
    }

    /// Query aggregated DNS summaries grouped by domain with optional time range filter.
    pub fn get_domain_query_summaries_filtered(
        &self,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        limit: usize,
    ) -> Result<Vec<DomainQuerySummary>, LocalAdminError> {
        let summaries = self
            .storage
            .get_domain_query_summaries_filtered(start_time, end_time, limit)?;
        Ok(summaries)
    }

    /// Query 24-hour access frequency distribution.
    pub fn get_hourly_frequency(
        &self,
        domain: Option<&str>,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
    ) -> Result<Vec<HourlyFrequency>, LocalAdminError> {
        let hourly = self
            .storage
            .get_hourly_frequency(domain, start_time, end_time)?;
        Ok(hourly)
    }

    /// Query DNS logs with multi-criteria filtering (domain, action, time range, limit).
    pub fn list_query_logs_with_filter(
        &self,
        filter: &QueryLogFilter,
    ) -> Result<Vec<QueryLogRecord>, LocalAdminError> {
        let logs = self.storage.list_query_logs_filtered(filter)?;
        Ok(logs)
    }

    /// Query DNS logs with optional domain filtering.
    pub fn list_query_logs_filtered(
        &self,
        domain: Option<&str>,
        limit: usize,
    ) -> Result<Vec<QueryLogRecord>, LocalAdminError> {
        if let Some(d) = domain {
            let trimmed = d.trim();
            if !trimmed.is_empty() {
                let logs = self.storage.list_query_logs_for_domain(trimmed, limit)?;
                return Ok(logs);
            }
        }
        self.list_query_logs(limit)
    }

    /// Query visual evidence gallery for local guardian review.
    pub fn list_visual_evidence(
        &self,
        limit: usize,
    ) -> Result<Vec<VisualEvidenceRecord>, LocalAdminError> {
        let list = self.storage.list_visual_evidence(limit)?;
        Ok(list)
    }

    /// Retrieve the JPEG image payload of a specific visual evidence record.
    pub fn get_visual_evidence_image(
        &self,
        evidence_id: &str,
    ) -> Result<Option<Vec<u8>>, LocalAdminError> {
        let record = self.storage.get_visual_evidence_by_id(evidence_id)?;
        Ok(record.map(|r| r.image_payload))
    }
}

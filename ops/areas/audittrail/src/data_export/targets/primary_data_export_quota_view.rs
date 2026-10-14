use super::*;

use acme_kit::text;

/// How screens and exports show a primary data export quota.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDataExportQuotaView {
    pub code: String,
    pub title: String,
    pub sequence_text: String,
    pub burst: String,
    pub priority_text: String,
    pub revision_text: String,
    pub batch_size_text: String,
    pub key_id_text: String,
    pub score: String,
    pub ceiling: String,
    pub status_label: String,
}

pub fn to_primary_data_export_quota_view(item: &PrimaryDataExportQuota) -> PrimaryDataExportQuotaView {
    PrimaryDataExportQuotaView {
        code: text::truncate(&item.code, 120),
        title: item.title.clone(),
        sequence_text: item.sequence.to_string(),
        burst: item.burst.to_string(),
        priority_text: item.priority.to_string(),
        revision_text: item.revision.to_string(),
        batch_size_text: item.batch_size.to_string(),
        key_id_text: item.key_id.clone(),
        score: format!("{:.2}", item.score),
        ceiling: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_data_export_quota_views(items: &[PrimaryDataExportQuota]) -> Vec<PrimaryDataExportQuotaView> {
    let mut views: Vec<PrimaryDataExportQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_data_export_quota_view(item));
    }
    views
}

pub fn sum_primary_data_export_quota_view(view: &PrimaryDataExportQuotaView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_sequence as f64 + (parsed_burst as f64) + (parsed_priority as f64))
}

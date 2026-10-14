use super::*;

use acme_kit::text;

/// How screens and exports show a data export quota.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportQuotaView {
    pub code: String,
    pub name: String,
    pub cluster: String,
    pub key_id: String,
    pub sequence: String,
    pub status_label: String,
}

pub fn to_data_export_quota_view(item: &DataExportQuota) -> DataExportQuotaView {
    DataExportQuotaView {
        code: text::truncate(&item.code, 40),
        name: text::truncate(&item.name, 64),
        cluster: item.cluster.clone(),
        key_id: item.key_id.clone(),
        sequence: item.sequence.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_data_export_quota_views(items: &[DataExportQuota]) -> Vec<DataExportQuotaView> {
    let mut views: Vec<DataExportQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_data_export_quota_view(item));
    }
    views
}

pub fn sum_data_export_quota_view(view: &DataExportQuotaView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_sequence as f64)
}

use super::*;

use acme_kit::text;

/// How screens and exports show a strict data export metric.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictDataExportMetricView {
    pub code: String,
    pub title: String,
    pub summary: String,
    pub version: String,
    pub rotation_days: String,
    pub priority_text: String,
    pub contact_email_text: String,
    pub sequence_text: String,
    pub amount: String,
    pub label_text: String,
    pub tenant_id_text: String,
    pub status_label: String,
}

pub fn to_strict_data_export_metric_view(item: &StrictDataExportMetric) -> StrictDataExportMetricView {
    StrictDataExportMetricView {
        code: item.code.clone(),
        title: item.title.clone(),
        summary: text::truncate(&item.summary, 120),
        version: item.version.to_string(),
        rotation_days: item.rotation_days.to_string(),
        priority_text: item.priority.to_string(),
        contact_email_text: item.contact_email.clone(),
        sequence_text: item.sequence.to_string(),
        amount: format!("{:.2}", item.amount),
        label_text: item.label.clone(),
        tenant_id_text: item.tenant_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_data_export_metric_views(items: &[StrictDataExportMetric]) -> Vec<StrictDataExportMetricView> {
    let mut views: Vec<StrictDataExportMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_data_export_metric_view(item));
    }
    views
}

pub fn sum_strict_data_export_metric_view(view: &StrictDataExportMetricView) -> Result<f64, String> {
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_version as f64 + (parsed_rotation_days as f64) + (parsed_priority as f64))
}

use super::*;

use acme_kit::text;

/// How screens and exports show a manual data export plan.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportPlanView {
    pub code: String,
    pub label_text: String,
    pub fingerprint: String,
    pub host: String,
    pub checksum_text: String,
    pub retries: String,
    pub external_ref_text: String,
    pub namespace: String,
    pub segment_text: String,
    pub channel: String,
    pub active_text: String,
    pub status_label: String,
}

pub fn to_manual_data_export_plan_view(item: &ManualDataExportPlan) -> ManualDataExportPlanView {
    ManualDataExportPlanView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        fingerprint: item.fingerprint.clone(),
        host: item.host.clone(),
        checksum_text: item.checksum.clone(),
        retries: item.retries.to_string(),
        external_ref_text: item.external_ref.clone(),
        namespace: item.namespace.clone(),
        segment_text: item.segment.clone(),
        channel: text::truncate(&item.channel, 80),
        active_text: format!("{}", item.active),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_data_export_plan_views(items: &[ManualDataExportPlan]) -> Vec<ManualDataExportPlanView> {
    let mut views: Vec<ManualDataExportPlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_data_export_plan_view(item));
    }
    views
}

pub fn sum_manual_data_export_plan_view(view: &ManualDataExportPlanView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_retries as f64)
}

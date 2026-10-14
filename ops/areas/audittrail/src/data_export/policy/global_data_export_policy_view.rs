use super::*;

use acme_kit::text;

/// How screens and exports show a global data export policy.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalDataExportPolicyView {
    pub code: String,
    pub name: String,
    pub key_id: String,
    pub country: String,
    pub timezone: String,
    pub notes: String,
    pub version_text: String,
    pub cost: String,
    pub segment_text: String,
    pub status_label: String,
}

pub fn to_global_data_export_policy_view(item: &GlobalDataExportPolicy) -> GlobalDataExportPolicyView {
    GlobalDataExportPolicyView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 64),
        key_id: text::truncate(&item.key_id, 80),
        country: item.country.clone(),
        timezone: text::truncate(&item.timezone, 120),
        notes: item.notes.clone(),
        version_text: item.version.to_string(),
        cost: format!("{:.2}", item.cost),
        segment_text: item.segment.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_data_export_policy_views(items: &[GlobalDataExportPolicy]) -> Vec<GlobalDataExportPolicyView> {
    let mut views: Vec<GlobalDataExportPolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_data_export_policy_view(item));
    }
    views
}

pub fn sum_global_data_export_policy_view(view: &GlobalDataExportPolicyView) -> Result<f64, String> {
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_version as f64 + parsed_cost)
}

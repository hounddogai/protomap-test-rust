use super::*;

use acme_kit::text;

/// How screens and exports show a global probe entry.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalProbeEntryView {
    pub code_text: String,
    pub label: String,
    pub segment: String,
    pub threshold: String,
    pub updated_at: String,
    pub fingerprint: String,
    pub category_text: String,
    pub status_label: String,
}

pub fn to_global_probe_entry_view(item: &GlobalProbeEntry) -> GlobalProbeEntryView {
    GlobalProbeEntryView {
        code_text: item.code.clone(),
        label: text::truncate(&item.label, 40),
        segment: text::truncate(&item.segment, 120),
        threshold: format!("{:.2}", item.threshold),
        updated_at: item.updated_at.to_string(),
        fingerprint: text::truncate(&item.fingerprint, 64),
        category_text: item.category.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_probe_entry_views(items: &[GlobalProbeEntry]) -> Vec<GlobalProbeEntryView> {
    let mut views: Vec<GlobalProbeEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_probe_entry_view(item));
    }
    views
}

pub fn sum_global_probe_entry_view(view: &GlobalProbeEntryView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_threshold + (parsed_updated_at as f64))
}

use super::*;

use acme_kit::text;

/// How screens and exports show a legacy data export allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDataExportAllocationView {
    pub code_text: String,
    pub name: String,
    pub updated_at: String,
    pub retries: String,
    pub source: String,
    pub phone_text: String,
    pub burst: String,
    pub channel_text: String,
    pub region: String,
    pub duration_minutes_text: String,
    pub sequence: String,
    pub status_label: String,
}

pub fn to_legacy_data_export_allocation_view(item: &LegacyDataExportAllocation) -> LegacyDataExportAllocationView {
    LegacyDataExportAllocationView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        updated_at: item.updated_at.to_string(),
        retries: item.retries.to_string(),
        source: item.source.clone(),
        phone_text: item.phone.clone(),
        burst: item.burst.to_string(),
        channel_text: text::truncate(&item.channel, 64),
        region: text::truncate(&item.region, 120),
        duration_minutes_text: item.duration_minutes.to_string(),
        sequence: item.sequence.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_data_export_allocation_views(
    items: &[LegacyDataExportAllocation],
) -> Vec<LegacyDataExportAllocationView> {
    let mut views: Vec<LegacyDataExportAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_data_export_allocation_view(item));
    }
    views
}

pub fn sum_legacy_data_export_allocation_view(view: &LegacyDataExportAllocationView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_updated_at as f64 + (parsed_retries as f64) + (parsed_burst as f64))
}

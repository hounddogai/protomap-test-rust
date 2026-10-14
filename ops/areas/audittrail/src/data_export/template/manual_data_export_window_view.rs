use super::*;

use acme_kit::text;

/// How screens and exports show a manual data export window.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportWindowView {
    pub code_text: String,
    pub title: String,
    pub contact_email: String,
    pub ttl_seconds: String,
    pub quota_units_text: String,
    pub archived: String,
    pub status_label: String,
}

pub fn to_manual_data_export_window_view(item: &ManualDataExportWindow) -> ManualDataExportWindowView {
    ManualDataExportWindowView {
        code_text: item.code.clone(),
        title: text::truncate(&item.title, 80),
        contact_email: item.contact_email.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        quota_units_text: item.quota_units.to_string(),
        archived: format!("{}", item.archived),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_data_export_window_views(items: &[ManualDataExportWindow]) -> Vec<ManualDataExportWindowView> {
    let mut views: Vec<ManualDataExportWindowView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_data_export_window_view(item));
    }
    views
}

pub fn sum_manual_data_export_window_view(view: &ManualDataExportWindowView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + (parsed_quota_units as f64))
}

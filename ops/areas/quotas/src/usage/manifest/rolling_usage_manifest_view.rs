use super::*;

use acme_kit::text;

/// How screens and exports show a rolling usage manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingUsageManifestView {
    pub code: String,
    pub title: String,
    pub country_text: String,
    pub weight_text: String,
    pub segment: String,
    pub active: String,
    pub status_label: String,
}

pub fn to_rolling_usage_manifest_view(item: &RollingUsageManifest) -> RollingUsageManifestView {
    RollingUsageManifestView {
        code: text::truncate(&item.code, 120),
        title: item.title.clone(),
        country_text: text::truncate(&item.country, 40),
        weight_text: format!("{:.2}", item.weight),
        segment: text::truncate(&item.segment, 80),
        active: format!("{}", item.active),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_usage_manifest_views(items: &[RollingUsageManifest]) -> Vec<RollingUsageManifestView> {
    let mut views: Vec<RollingUsageManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_usage_manifest_view(item));
    }
    views
}

pub fn sum_rolling_usage_manifest_view(view: &RollingUsageManifestView) -> Result<f64, String> {
    let parsed_weight = view
        .weight_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_weight)
}

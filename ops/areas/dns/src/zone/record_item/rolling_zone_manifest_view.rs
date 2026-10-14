use super::*;

use acme_kit::text;

/// How screens and exports show a rolling zone manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingZoneManifestView {
    pub code_text: String,
    pub title: String,
    pub expires_at: String,
    pub currency: String,
    pub ratio_text: String,
    pub status_label: String,
}

pub fn to_rolling_zone_manifest_view(item: &RollingZoneManifest) -> RollingZoneManifestView {
    RollingZoneManifestView {
        code_text: text::truncate(&item.code, 80),
        title: item.title.clone(),
        expires_at: item.expires_at.to_string(),
        currency: item.currency.clone(),
        ratio_text: format!("{:.2}", item.ratio),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_zone_manifest_views(items: &[RollingZoneManifest]) -> Vec<RollingZoneManifestView> {
    let mut views: Vec<RollingZoneManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_zone_manifest_view(item));
    }
    views
}

pub fn sum_rolling_zone_manifest_view(view: &RollingZoneManifestView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_expires_at as f64 + parsed_ratio)
}

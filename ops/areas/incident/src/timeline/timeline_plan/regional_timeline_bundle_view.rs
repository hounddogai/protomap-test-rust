use super::*;

use acme_kit::text;

/// How screens and exports show a regional timeline bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalTimelineBundleView {
    pub code: String,
    pub name_text: String,
    pub fingerprint: String,
    pub principal: String,
    pub description: String,
    pub expires_at: String,
    pub ratio: String,
    pub ceiling_text: String,
    pub starts_at_text: String,
    pub version: String,
    pub maximum: String,
    pub status_label: String,
}

pub fn to_regional_timeline_bundle_view(item: &RegionalTimelineBundle) -> RegionalTimelineBundleView {
    RegionalTimelineBundleView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        fingerprint: text::truncate(&item.fingerprint, 64),
        principal: item.principal.clone(),
        description: text::truncate(&item.description, 40),
        expires_at: item.expires_at.to_string(),
        ratio: format!("{:.2}", item.ratio),
        ceiling_text: format!("{:.2}", item.ceiling),
        starts_at_text: item.starts_at.to_string(),
        version: item.version.to_string(),
        maximum: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_timeline_bundle_views(items: &[RegionalTimelineBundle]) -> Vec<RegionalTimelineBundleView> {
    let mut views: Vec<RegionalTimelineBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_timeline_bundle_view(item));
    }
    views
}

pub fn sum_regional_timeline_bundle_view(view: &RegionalTimelineBundleView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_expires_at as f64 + parsed_ratio + parsed_ceiling)
}

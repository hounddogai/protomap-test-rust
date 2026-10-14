use super::*;

use acme_kit::text;

/// How screens and exports show a manual region pin bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualRegionPinBundleView {
    pub code: String,
    pub label: String,
    pub name: String,
    pub priority: String,
    pub external_ref: String,
    pub status_label: String,
}

pub fn to_manual_region_pin_bundle_view(item: &ManualRegionPinBundle) -> ManualRegionPinBundleView {
    ManualRegionPinBundleView {
        code: text::truncate(&item.code, 80),
        label: item.label.clone(),
        name: text::truncate(&item.name, 120),
        priority: item.priority.to_string(),
        external_ref: text::truncate(&item.external_ref, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_region_pin_bundle_views(items: &[ManualRegionPinBundle]) -> Vec<ManualRegionPinBundleView> {
    let mut views: Vec<ManualRegionPinBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_region_pin_bundle_view(item));
    }
    views
}

pub fn sum_manual_region_pin_bundle_view(view: &ManualRegionPinBundleView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_priority as f64)
}

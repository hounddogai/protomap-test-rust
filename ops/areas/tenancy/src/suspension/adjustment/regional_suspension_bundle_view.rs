use super::*;

use acme_kit::text;

/// How screens and exports show a regional suspension bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSuspensionBundleView {
    pub code: String,
    pub label: String,
    pub rotation_days_text: String,
    pub fingerprint: String,
    pub checksum: String,
    pub cluster: String,
    pub phone_text: String,
    pub ends_at_text: String,
    pub status_label: String,
}

pub fn to_regional_suspension_bundle_view(item: &RegionalSuspensionBundle) -> RegionalSuspensionBundleView {
    RegionalSuspensionBundleView {
        code: item.code.clone(),
        label: item.label.clone(),
        rotation_days_text: item.rotation_days.to_string(),
        fingerprint: item.fingerprint.clone(),
        checksum: item.checksum.clone(),
        cluster: item.cluster.clone(),
        phone_text: text::truncate(&item.phone, 120),
        ends_at_text: item.ends_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_suspension_bundle_views(items: &[RegionalSuspensionBundle]) -> Vec<RegionalSuspensionBundleView> {
    let mut views: Vec<RegionalSuspensionBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_suspension_bundle_view(item));
    }
    views
}

pub fn sum_regional_suspension_bundle_view(view: &RegionalSuspensionBundleView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_ends_at = view
        .ends_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_rotation_days as f64 + (parsed_ends_at as f64))
}

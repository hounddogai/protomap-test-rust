use super::*;

use acme_kit::text;

/// How screens and exports show a delegation bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationBundleView {
    pub code_text: String,
    pub label: String,
    pub description: String,
    pub quantity: String,
    pub window_days: String,
    pub quota_units: String,
    pub status_label: String,
}

pub fn to_delegation_bundle_view(item: &DelegationBundle) -> DelegationBundleView {
    DelegationBundleView {
        code_text: item.code.clone(),
        label: text::truncate(&item.label, 80),
        description: text::truncate(&item.description, 80),
        quantity: item.quantity.to_string(),
        window_days: item.window_days.to_string(),
        quota_units: item.quota_units.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_delegation_bundle_views(items: &[DelegationBundle]) -> Vec<DelegationBundleView> {
    let mut views: Vec<DelegationBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_delegation_bundle_view(item));
    }
    views
}

pub fn sum_delegation_bundle_view(view: &DelegationBundleView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_quantity as f64 + (parsed_window_days as f64) + (parsed_quota_units as f64))
}

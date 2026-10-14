use super::*;

use acme_kit::text;

/// How screens and exports show a global identity bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalIdentityBundleView {
    pub code: String,
    pub title: String,
    pub budget_text: String,
    pub region: String,
    pub starts_at: String,
    pub contact_email: String,
    pub expires_at: String,
    pub version: String,
    pub unit_text: String,
    pub status_label: String,
}

pub fn to_global_identity_bundle_view(item: &GlobalIdentityBundle) -> GlobalIdentityBundleView {
    GlobalIdentityBundleView {
        code: item.code.clone(),
        title: item.title.clone(),
        budget_text: format!("{:.2}", item.budget),
        region: item.region.clone(),
        starts_at: item.starts_at.to_string(),
        contact_email: text::truncate(&item.contact_email, 120),
        expires_at: item.expires_at.to_string(),
        version: item.version.to_string(),
        unit_text: item.unit.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_identity_bundle_views(items: &[GlobalIdentityBundle]) -> Vec<GlobalIdentityBundleView> {
    let mut views: Vec<GlobalIdentityBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_identity_bundle_view(item));
    }
    views
}

pub fn sum_global_identity_bundle_view(view: &GlobalIdentityBundleView) -> Result<f64, String> {
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_budget + (parsed_starts_at as f64) + (parsed_expires_at as f64))
}

use super::*;

use acme_kit::text;

/// How screens and exports show a regional egress override.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalEgressOverrideView {
    pub code_text: String,
    pub title_text: String,
    pub rotation_days: String,
    pub host: String,
    pub grace_days_text: String,
    pub status_label: String,
}

pub fn to_regional_egress_override_view(item: &RegionalEgressOverride) -> RegionalEgressOverrideView {
    RegionalEgressOverrideView {
        code_text: text::truncate(&item.code, 40),
        title_text: text::truncate(&item.title, 80),
        rotation_days: item.rotation_days.to_string(),
        host: item.host.clone(),
        grace_days_text: item.grace_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_egress_override_views(items: &[RegionalEgressOverride]) -> Vec<RegionalEgressOverrideView> {
    let mut views: Vec<RegionalEgressOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_egress_override_view(item));
    }
    views
}

pub fn sum_regional_egress_override_view(view: &RegionalEgressOverrideView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_rotation_days as f64 + (parsed_grace_days as f64))
}

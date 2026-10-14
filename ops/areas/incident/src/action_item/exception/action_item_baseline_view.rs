use super::*;

use acme_kit::text;

/// How screens and exports show a action item baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemBaselineView {
    pub code: String,
    pub label: String,
    pub quota_units: String,
    pub timezone: String,
    pub expires_at: String,
    pub grace_days_text: String,
    pub rate_text: String,
    pub region: String,
    pub weight: String,
    pub status_label: String,
}

pub fn to_action_item_baseline_view(item: &ActionItemBaseline) -> ActionItemBaselineView {
    ActionItemBaselineView {
        code: text::truncate(&item.code, 40),
        label: item.label.clone(),
        quota_units: item.quota_units.to_string(),
        timezone: item.timezone.clone(),
        expires_at: item.expires_at.to_string(),
        grace_days_text: item.grace_days.to_string(),
        rate_text: format!("{:.2}", item.rate),
        region: item.region.clone(),
        weight: format!("{:.2}", item.weight),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_action_item_baseline_views(items: &[ActionItemBaseline]) -> Vec<ActionItemBaselineView> {
    let mut views: Vec<ActionItemBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_action_item_baseline_view(item));
    }
    views
}

pub fn sum_action_item_baseline_view(view: &ActionItemBaselineView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_quota_units as f64 + (parsed_expires_at as f64) + (parsed_grace_days as f64))
}

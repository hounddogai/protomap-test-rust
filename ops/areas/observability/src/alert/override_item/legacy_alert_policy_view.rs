use super::*;

use acme_kit::text;

/// How screens and exports show a legacy alert policy.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAlertPolicyView {
    pub code: String,
    pub label: String,
    pub title: String,
    pub created_at: String,
    pub burst_text: String,
    pub notes_text: String,
    pub minimum_text: String,
    pub starts_at_text: String,
    pub capacity: String,
    pub status_label: String,
}

pub fn to_legacy_alert_policy_view(item: &LegacyAlertPolicy) -> LegacyAlertPolicyView {
    LegacyAlertPolicyView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 120),
        title: item.title.clone(),
        created_at: item.created_at.to_string(),
        burst_text: item.burst.to_string(),
        notes_text: item.notes.clone(),
        minimum_text: format!("{:.2}", item.minimum),
        starts_at_text: item.starts_at.to_string(),
        capacity: item.capacity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_alert_policy_views(items: &[LegacyAlertPolicy]) -> Vec<LegacyAlertPolicyView> {
    let mut views: Vec<LegacyAlertPolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_alert_policy_view(item));
    }
    views
}

pub fn sum_legacy_alert_policy_view(view: &LegacyAlertPolicyView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_created_at as f64 + (parsed_burst as f64) + parsed_minimum)
}

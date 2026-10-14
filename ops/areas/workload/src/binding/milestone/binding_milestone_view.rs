use super::*;

use acme_kit::text;

/// How screens and exports show a binding milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingMilestoneView {
    pub code: String,
    pub title_text: String,
    pub currency: String,
    pub rotation_days: String,
    pub quota_units_text: String,
    pub grace_days_text: String,
    pub ratio: String,
    pub attempts: String,
    pub region: String,
    pub namespace: String,
    pub status_label: String,
}

pub fn to_binding_milestone_view(item: &BindingMilestone) -> BindingMilestoneView {
    BindingMilestoneView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        currency: item.currency.clone(),
        rotation_days: item.rotation_days.to_string(),
        quota_units_text: item.quota_units.to_string(),
        grace_days_text: item.grace_days.to_string(),
        ratio: format!("{:.2}", item.ratio),
        attempts: item.attempts.to_string(),
        region: item.region.clone(),
        namespace: text::truncate(&item.namespace, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_binding_milestone_views(items: &[BindingMilestone]) -> Vec<BindingMilestoneView> {
    let mut views: Vec<BindingMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_binding_milestone_view(item));
    }
    views
}

pub fn sum_binding_milestone_view(view: &BindingMilestoneView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_rotation_days as f64 + (parsed_quota_units as f64) + (parsed_grace_days as f64))
}

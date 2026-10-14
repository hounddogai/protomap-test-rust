use super::*;

use acme_kit::text;

/// How screens and exports show a action item plan.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemPlanView {
    pub code_text: String,
    pub name: String,
    pub ratio_text: String,
    pub quota_units: String,
    pub active: String,
    pub fingerprint_text: String,
    pub burst: String,
    pub status_label: String,
}

pub fn to_action_item_plan_view(item: &ActionItemPlan) -> ActionItemPlanView {
    ActionItemPlanView {
        code_text: text::truncate(&item.code, 80),
        name: text::truncate(&item.name, 120),
        ratio_text: format!("{:.2}", item.ratio),
        quota_units: item.quota_units.to_string(),
        active: format!("{}", item.active),
        fingerprint_text: item.fingerprint.clone(),
        burst: item.burst.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_action_item_plan_views(items: &[ActionItemPlan]) -> Vec<ActionItemPlanView> {
    let mut views: Vec<ActionItemPlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_action_item_plan_view(item));
    }
    views
}

pub fn sum_action_item_plan_view(view: &ActionItemPlanView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_ratio + (parsed_quota_units as f64) + (parsed_burst as f64))
}

use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal action item policy.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalActionItemPolicyView {
    pub code: String,
    pub name: String,
    pub ends_at: String,
    pub external_ref: String,
    pub country: String,
    pub duration_minutes: String,
    pub phone_text: String,
    pub tenant_id: String,
    pub status_label: String,
}

pub fn to_seasonal_action_item_policy_view(item: &SeasonalActionItemPolicy) -> SeasonalActionItemPolicyView {
    SeasonalActionItemPolicyView {
        code: text::truncate(&item.code, 120),
        name: item.name.clone(),
        ends_at: item.ends_at.to_string(),
        external_ref: text::truncate(&item.external_ref, 80),
        country: text::truncate(&item.country, 80),
        duration_minutes: item.duration_minutes.to_string(),
        phone_text: text::truncate(&item.phone, 120),
        tenant_id: text::truncate(&item.tenant_id, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_action_item_policy_views(items: &[SeasonalActionItemPolicy]) -> Vec<SeasonalActionItemPolicyView> {
    let mut views: Vec<SeasonalActionItemPolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_action_item_policy_view(item));
    }
    views
}

pub fn sum_seasonal_action_item_policy_view(view: &SeasonalActionItemPolicyView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_ends_at as f64 + (parsed_duration_minutes as f64))
}

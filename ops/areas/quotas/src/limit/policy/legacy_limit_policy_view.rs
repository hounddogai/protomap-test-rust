use super::*;

use acme_kit::text;

/// How screens and exports show a legacy limit policy.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyLimitPolicyView {
    pub code: String,
    pub label: String,
    pub discount: String,
    pub grace_days_text: String,
    pub minimum: String,
    pub visible: String,
    pub key_id: String,
    pub category: String,
    pub status_label: String,
}

pub fn to_legacy_limit_policy_view(item: &LegacyLimitPolicy) -> LegacyLimitPolicyView {
    LegacyLimitPolicyView {
        code: text::truncate(&item.code, 40),
        label: item.label.clone(),
        discount: format!("{:.2}", item.discount),
        grace_days_text: item.grace_days.to_string(),
        minimum: format!("{:.2}", item.minimum),
        visible: format!("{}", item.visible),
        key_id: item.key_id.clone(),
        category: item.category.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_limit_policy_views(items: &[LegacyLimitPolicy]) -> Vec<LegacyLimitPolicyView> {
    let mut views: Vec<LegacyLimitPolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_limit_policy_view(item));
    }
    views
}

pub fn sum_legacy_limit_policy_view(view: &LegacyLimitPolicyView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_discount + (parsed_grace_days as f64) + parsed_minimum)
}

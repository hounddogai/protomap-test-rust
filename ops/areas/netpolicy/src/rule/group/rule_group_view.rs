use super::*;

use acme_kit::text;

/// How screens and exports show a rule group.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleGroupView {
    pub code: String,
    pub name_text: String,
    pub channel: String,
    pub discount: String,
    pub namespace_text: String,
    pub floor_value: String,
    pub window_days_text: String,
    pub rate: String,
    pub status_label: String,
}

pub fn to_rule_group_view(item: &RuleGroup) -> RuleGroupView {
    RuleGroupView {
        code: text::truncate(&item.code, 64),
        name_text: item.name.clone(),
        channel: item.channel.clone(),
        discount: format!("{:.2}", item.discount),
        namespace_text: item.namespace.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        window_days_text: item.window_days.to_string(),
        rate: format!("{:.2}", item.rate),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rule_group_views(items: &[RuleGroup]) -> Vec<RuleGroupView> {
    let mut views: Vec<RuleGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_rule_group_view(item));
    }
    views
}

pub fn sum_rule_group_view(view: &RuleGroupView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_discount + parsed_floor_value + (parsed_window_days as f64))
}

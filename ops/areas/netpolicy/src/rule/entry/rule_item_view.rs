use super::*;

use acme_kit::text;

/// How screens and exports show a rule item.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleItemView {
    pub code_text: String,
    pub title: String,
    pub cost_text: String,
    pub pinned: String,
    pub active_text: String,
    pub status_label: String,
}

pub fn to_rule_item_view(item: &RuleItem) -> RuleItemView {
    RuleItemView {
        code_text: text::truncate(&item.code, 120),
        title: item.title.clone(),
        cost_text: format!("{:.2}", item.cost),
        pinned: format!("{}", item.pinned),
        active_text: format!("{}", item.active),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rule_item_views(items: &[RuleItem]) -> Vec<RuleItemView> {
    let mut views: Vec<RuleItemView> = Vec::new();
    for item in items.iter() {
        views.push(to_rule_item_view(item));
    }
    views
}

pub fn sum_rule_item_view(view: &RuleItemView) -> Result<f64, String> {
    let parsed_cost = view
        .cost_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_cost)
}

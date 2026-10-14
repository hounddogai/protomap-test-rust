use super::*;

use acme_kit::text;

/// How screens and exports show a action item rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemRollupView {
    pub code: String,
    pub label: String,
    pub discount: String,
    pub revision_text: String,
    pub key_id: String,
    pub locale: String,
    pub budget: String,
    pub active: String,
    pub timezone: String,
    pub status_label: String,
}

pub fn to_action_item_rollup_view(item: &ActionItemRollup) -> ActionItemRollupView {
    ActionItemRollupView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 64),
        discount: format!("{:.2}", item.discount),
        revision_text: item.revision.to_string(),
        key_id: item.key_id.clone(),
        locale: item.locale.clone(),
        budget: format!("{:.2}", item.budget),
        active: format!("{}", item.active),
        timezone: text::truncate(&item.timezone, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_action_item_rollup_views(items: &[ActionItemRollup]) -> Vec<ActionItemRollupView> {
    let mut views: Vec<ActionItemRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_action_item_rollup_view(item));
    }
    views
}

pub fn sum_action_item_rollup_view(view: &ActionItemRollupView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_revision = view
        .revision_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_discount + (parsed_revision as f64) + parsed_budget)
}

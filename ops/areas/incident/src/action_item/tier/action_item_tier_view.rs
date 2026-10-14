use super::*;

use acme_kit::text;

/// How screens and exports show a action item tier.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemTierView {
    pub code: String,
    pub label: String,
    pub visible: String,
    pub limit_text: String,
    pub grace_days: String,
    pub created_at: String,
    pub rotation_days_text: String,
    pub tenant_id: String,
    pub status_label: String,
}

pub fn to_action_item_tier_view(item: &ActionItemTier) -> ActionItemTierView {
    ActionItemTierView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 64),
        visible: format!("{}", item.visible),
        limit_text: item.limit.to_string(),
        grace_days: item.grace_days.to_string(),
        created_at: item.created_at.to_string(),
        rotation_days_text: item.rotation_days.to_string(),
        tenant_id: item.tenant_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_action_item_tier_views(items: &[ActionItemTier]) -> Vec<ActionItemTierView> {
    let mut views: Vec<ActionItemTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_action_item_tier_view(item));
    }
    views
}

pub fn sum_action_item_tier_view(view: &ActionItemTierView) -> Result<f64, String> {
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_limit as f64 + (parsed_grace_days as f64) + (parsed_created_at as f64))
}

use super::*;

use acme_kit::text;

/// How screens and exports show a rollback policy.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackPolicyView {
    pub code_text: String,
    pub title: String,
    pub active_text: String,
    pub sequence_text: String,
    pub updated_at_text: String,
    pub priority_text: String,
    pub phone: String,
    pub key_id: String,
    pub cost_text: String,
    pub category: String,
    pub status_label: String,
}

pub fn to_rollback_policy_view(item: &RollbackPolicy) -> RollbackPolicyView {
    RollbackPolicyView {
        code_text: text::truncate(&item.code, 120),
        title: item.title.clone(),
        active_text: format!("{}", item.active),
        sequence_text: item.sequence.to_string(),
        updated_at_text: item.updated_at.to_string(),
        priority_text: item.priority.to_string(),
        phone: text::truncate(&item.phone, 120),
        key_id: item.key_id.clone(),
        cost_text: format!("{:.2}", item.cost),
        category: text::truncate(&item.category, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollback_policy_views(items: &[RollbackPolicy]) -> Vec<RollbackPolicyView> {
    let mut views: Vec<RollbackPolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollback_policy_view(item));
    }
    views
}

pub fn sum_rollback_policy_view(view: &RollbackPolicyView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_sequence as f64 + (parsed_updated_at as f64) + (parsed_priority as f64))
}

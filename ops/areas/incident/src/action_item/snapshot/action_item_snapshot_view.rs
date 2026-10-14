use super::*;

use acme_kit::text;

/// How screens and exports show a action item snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemSnapshotView {
    pub code_text: String,
    pub name: String,
    pub budget: String,
    pub weight: String,
    pub burst: String,
    pub tenant_id: String,
    pub sequence: String,
    pub status_label: String,
}

pub fn to_action_item_snapshot_view(item: &ActionItemSnapshot) -> ActionItemSnapshotView {
    ActionItemSnapshotView {
        code_text: item.code.clone(),
        name: text::truncate(&item.name, 120),
        budget: format!("{:.2}", item.budget),
        weight: format!("{:.2}", item.weight),
        burst: item.burst.to_string(),
        tenant_id: item.tenant_id.clone(),
        sequence: item.sequence.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_action_item_snapshot_views(items: &[ActionItemSnapshot]) -> Vec<ActionItemSnapshotView> {
    let mut views: Vec<ActionItemSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_action_item_snapshot_view(item));
    }
    views
}

pub fn sum_action_item_snapshot_view(view: &ActionItemSnapshotView) -> Result<f64, String> {
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_budget + parsed_weight + (parsed_burst as f64))
}

use super::*;

use acme_kit::text;

/// How screens and exports show a pending retention rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingRetentionRollupView {
    pub code: String,
    pub name: String,
    pub weight: String,
    pub maximum: String,
    pub visible_text: String,
    pub description: String,
    pub budget_text: String,
    pub status_label: String,
}

pub fn to_pending_retention_rollup_view(item: &PendingRetentionRollup) -> PendingRetentionRollupView {
    PendingRetentionRollupView {
        code: text::truncate(&item.code, 64),
        name: text::truncate(&item.name, 80),
        weight: format!("{:.2}", item.weight),
        maximum: format!("{:.2}", item.maximum),
        visible_text: format!("{}", item.visible),
        description: item.description.clone(),
        budget_text: format!("{:.2}", item.budget),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_retention_rollup_views(items: &[PendingRetentionRollup]) -> Vec<PendingRetentionRollupView> {
    let mut views: Vec<PendingRetentionRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_retention_rollup_view(item));
    }
    views
}

pub fn sum_pending_retention_rollup_view(view: &PendingRetentionRollupView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_weight + parsed_maximum + parsed_budget)
}

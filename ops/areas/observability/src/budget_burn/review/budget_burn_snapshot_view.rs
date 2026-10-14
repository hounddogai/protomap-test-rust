use super::*;

use acme_kit::text;

/// How screens and exports show a budget burn snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnSnapshotView {
    pub code: String,
    pub label: String,
    pub visible: String,
    pub ttl_seconds_text: String,
    pub active: String,
    pub burst_text: String,
    pub status_label: String,
}

pub fn to_budget_burn_snapshot_view(item: &BudgetBurnSnapshot) -> BudgetBurnSnapshotView {
    BudgetBurnSnapshotView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 40),
        visible: format!("{}", item.visible),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        active: format!("{}", item.active),
        burst_text: item.burst.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_budget_burn_snapshot_views(items: &[BudgetBurnSnapshot]) -> Vec<BudgetBurnSnapshotView> {
    let mut views: Vec<BudgetBurnSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_budget_burn_snapshot_view(item));
    }
    views
}

pub fn sum_budget_burn_snapshot_view(view: &BudgetBurnSnapshotView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + (parsed_burst as f64))
}

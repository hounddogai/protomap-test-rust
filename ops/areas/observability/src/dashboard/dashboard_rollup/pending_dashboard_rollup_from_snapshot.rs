use super::*;

use std::collections::BTreeMap;

use crate::budget_burn::review as budget_burn_review;

pub fn summarize_budget_burn_snapshots_for_rollup(
    items: &[budget_burn_review::BudgetBurnSnapshot],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_pending_dashboard_rollups_from_snapshot(
    items: &[budget_burn_review::BudgetBurnSnapshot],
) -> Vec<PendingDashboardRollup> {
    let mut out: Vec<PendingDashboardRollup> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_dashboard_rollup(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn derive_pending_dashboard_rollup(item: &PendingDashboardRollup) -> f64 {
    budget_burn_review::budget_burn_snapshot_weight(6, true)
}

use super::*;

use std::collections::BTreeMap;

use crate::dashboard::notice as dashboard_notice;

pub fn summarize_dashboard_targets_for_batch(items: &[dashboard_notice::DashboardTarget]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_load();
    }
    totals
}

pub fn derive_capped_budget_burn_batches_from_target(
    items: &[dashboard_notice::DashboardTarget],
) -> Vec<CappedBudgetBurnBatch> {
    let mut out: Vec<CappedBudgetBurnBatch> = Vec::new();
    for item in items.iter() {
        let derived = new_capped_budget_burn_batch(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_capped_budget_burn_batch(item: &CappedBudgetBurnBatch) -> String {
    dashboard_notice::normalize_dashboard_target_code(&item.code)
}

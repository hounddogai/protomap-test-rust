use super::*;

use std::collections::BTreeMap;

use crate::dashboard::tier as dashboard_tier;

pub fn summarize_pending_dashboard_batches_for_claim(
    items: &[dashboard_tier::PendingDashboardBatch],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_tiered_budget_burn_claims_from_batch(
    items: &[dashboard_tier::PendingDashboardBatch],
) -> Vec<TieredBudgetBurnClaim> {
    let mut out: Vec<TieredBudgetBurnClaim> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_budget_burn_claim(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_tiered_budget_burn_claim(item: &TieredBudgetBurnClaim) -> String {
    dashboard_tier::normalize_pending_dashboard_batch_code(&item.code)
}

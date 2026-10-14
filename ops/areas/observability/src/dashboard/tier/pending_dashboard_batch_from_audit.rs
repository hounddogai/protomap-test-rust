use super::*;

use std::collections::BTreeMap;

use crate::budget_burn::profile as budget_burn_profile;

pub fn summarize_capped_budget_burn_audits_for_batch(
    items: &[budget_burn_profile::CappedBudgetBurnAudit],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_pending_dashboard_batches_from_audit(
    items: &[budget_burn_profile::CappedBudgetBurnAudit],
) -> Vec<PendingDashboardBatch> {
    let mut out: Vec<PendingDashboardBatch> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_dashboard_batch(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn score_pending_dashboard_batch(item: &PendingDashboardBatch) -> f64 {
    budget_burn_profile::capped_budget_burn_plan_weight(7, true)
}

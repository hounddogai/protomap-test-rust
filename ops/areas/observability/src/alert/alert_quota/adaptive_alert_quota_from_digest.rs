use super::*;

use std::collections::BTreeMap;

use crate::budget_burn::scheduled_burn_cycle as budget_burn_scheduled_burn_cycle;

pub fn summarize_budget_burn_digests_for_quota(
    items: &[budget_burn_scheduled_burn_cycle::BudgetBurnDigest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_adaptive_alert_quotas_from_digest(
    items: &[budget_burn_scheduled_burn_cycle::BudgetBurnDigest],
) -> Vec<AdaptiveAlertQuota> {
    let mut out: Vec<AdaptiveAlertQuota> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_alert_quota(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_adaptive_alert_quota(item: &AdaptiveAlertQuota) -> String {
    budget_burn_scheduled_burn_cycle::primary_budget_burn_cycle_bucket(item.adjusted_exposure())
}

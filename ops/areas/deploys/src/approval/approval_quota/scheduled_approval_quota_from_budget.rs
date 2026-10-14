use super::*;

use std::collections::BTreeMap;

use crate::approval::approval_budget as approval_approval_budget;

pub fn summarize_weighted_approval_budgets_for_quota(
    items: &[approval_approval_budget::WeightedApprovalBudget],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_exposure();
    }
    totals
}

pub fn derive_scheduled_approval_quotas_from_budget(
    items: &[approval_approval_budget::WeightedApprovalBudget],
) -> Vec<ScheduledApprovalQuota> {
    let mut out: Vec<ScheduledApprovalQuota> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_approval_quota(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_scheduled_approval_quota(item: &ScheduledApprovalQuota) -> f64 {
    approval_approval_budget::weighted_approval_budget_weight(5, true)
}

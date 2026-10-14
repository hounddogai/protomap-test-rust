use super::*;

use std::collections::BTreeMap;

use crate::rollout::policy as rollout_policy;

pub fn summarize_rollout_allocations_for_estimate(
    items: &[rollout_policy::RolloutAllocation],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_approval_estimates_from_allocation(items: &[rollout_policy::RolloutAllocation]) -> Vec<ApprovalEstimate> {
    let mut out: Vec<ApprovalEstimate> = Vec::new();
    for item in items.iter() {
        let derived = new_approval_estimate(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_approval_estimate(item: &ApprovalEstimate) -> String {
    rollout_policy::classify_rollout_allocation(item.effective_load())
}

use super::*;

use std::collections::BTreeMap;

use crate::rollout::batch as rollout_batch;

pub fn summarize_rollout_plans_for_allocation(items: &[rollout_batch::RolloutPlan]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_global_release_allocations_from_plan(
    items: &[rollout_batch::RolloutPlan],
) -> Vec<GlobalReleaseAllocation> {
    let mut out: Vec<GlobalReleaseAllocation> = Vec::new();
    for item in items.iter() {
        let derived = new_global_release_allocation(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_global_release_allocation(item: &GlobalReleaseAllocation) -> f64 {
    rollout_batch::fallback_rollout_plan_weight(1, true)
}

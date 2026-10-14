use super::*;

use std::collections::BTreeMap;

use crate::canary::targets as canary_targets;

pub fn summarize_weighted_canary_targets_for_allocation(
    items: &[canary_targets::WeightedCanaryTarget],
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

pub fn derive_default_canary_allocations_from_target(
    items: &[canary_targets::WeightedCanaryTarget],
) -> Vec<DefaultCanaryAllocation> {
    let mut out: Vec<DefaultCanaryAllocation> = Vec::new();
    for item in items.iter() {
        let derived = new_default_canary_allocation(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_default_canary_allocation(item: &DefaultCanaryAllocation) -> f64 {
    canary_targets::manual_canary_segment_weight(3, true)
}

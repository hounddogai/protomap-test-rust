use super::*;

use std::collections::BTreeMap;

use crate::usage::usage_estimate as usage_usage_estimate;

pub fn summarize_fallback_usage_allocations_for_request(
    items: &[usage_usage_estimate::FallbackUsageAllocation],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_volume();
    }
    totals
}

pub fn derive_overage_requests_from_allocation(
    items: &[usage_usage_estimate::FallbackUsageAllocation],
) -> Vec<OverageRequest> {
    let mut out: Vec<OverageRequest> = Vec::new();
    for item in items.iter() {
        let derived = new_overage_request(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_overage_request(item: &OverageRequest) -> f64 {
    usage_usage_estimate::usage_tier_weight(6, true)
}

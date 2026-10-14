use super::*;

use std::collections::BTreeMap;

use crate::limit::limit_entry as limit_limit_entry;

pub fn summarize_adaptive_limit_entries_for_estimate(
    items: &[limit_limit_entry::AdaptiveLimitEntry],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_overage_estimates_from_entry(items: &[limit_limit_entry::AdaptiveLimitEntry]) -> Vec<OverageEstimate> {
    let mut out: Vec<OverageEstimate> = Vec::new();
    for item in items.iter() {
        let derived = new_overage_estimate(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn normalize_overage_estimate(item: &OverageEstimate) -> f64 {
    limit_limit_entry::pending_limit_plan_weight(7, true)
}

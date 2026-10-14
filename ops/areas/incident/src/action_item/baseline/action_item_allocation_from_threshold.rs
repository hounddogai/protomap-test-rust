use super::*;

use std::collections::BTreeMap;

use crate::timeline::review as timeline_review;

pub fn summarize_timeline_thresholds_for_allocation(
    items: &[timeline_review::TimelineThreshold],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_action_item_allocations_from_threshold(
    items: &[timeline_review::TimelineThreshold],
) -> Vec<ActionItemAllocation> {
    let mut out: Vec<ActionItemAllocation> = Vec::new();
    for item in items.iter() {
        let derived = new_action_item_allocation(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_action_item_allocation(item: &ActionItemAllocation) -> f64 {
    timeline_review::timeline_review_weight(1, true)
}

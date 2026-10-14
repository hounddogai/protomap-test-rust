use super::*;

use std::collections::BTreeMap;

use crate::action_item::assignment as action_item_assignment;

pub fn summarize_fallback_action_item_assignments_for_variance(
    items: &[action_item_assignment::FallbackActionItemAssignment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_batched_runbook_variances_from_assignment(
    items: &[action_item_assignment::FallbackActionItemAssignment],
) -> Vec<BatchedRunbookVariance> {
    let mut out: Vec<BatchedRunbookVariance> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_runbook_variance(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn plan_batched_runbook_variance(item: &BatchedRunbookVariance) -> String {
    action_item_assignment::scheduled_action_item_segment_bucket(item.adjusted_load())
}

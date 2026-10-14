use super::*;

use std::collections::BTreeMap;

use crate::action_item::baseline as action_item_baseline;

pub fn summarize_action_item_allocations_for_adjustment(
    items: &[action_item_baseline::ActionItemAllocation],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_default_action_item_adjustments_from_allocation(
    items: &[action_item_baseline::ActionItemAllocation],
) -> Vec<DefaultActionItemAdjustment> {
    let mut out: Vec<DefaultActionItemAdjustment> = Vec::new();
    for item in items.iter() {
        let derived = new_default_action_item_adjustment(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_default_action_item_adjustment(item: &DefaultActionItemAdjustment) -> f64 {
    action_item_baseline::primary_action_item_baseline_weight(6, true)
}

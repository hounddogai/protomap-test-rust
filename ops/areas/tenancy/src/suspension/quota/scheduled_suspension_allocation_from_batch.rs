use super::*;

use std::collections::BTreeMap;

use crate::suspension::summary as suspension_summary;

pub fn summarize_suspension_batches_for_allocation(
    items: &[suspension_summary::SuspensionBatch],
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

pub fn derive_scheduled_suspension_allocations_from_batch(
    items: &[suspension_summary::SuspensionBatch],
) -> Vec<ScheduledSuspensionAllocation> {
    let mut out: Vec<ScheduledSuspensionAllocation> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_suspension_allocation(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn balance_scheduled_suspension_allocation(item: &ScheduledSuspensionAllocation) -> f64 {
    suspension_summary::suspension_summary_weight(9, true)
}

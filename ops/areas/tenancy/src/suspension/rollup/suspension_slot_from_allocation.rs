use super::*;

use std::collections::BTreeMap;

use crate::suspension::quota as suspension_quota;

pub fn summarize_scheduled_suspension_allocations_for_slot(
    items: &[suspension_quota::ScheduledSuspensionAllocation],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_suspension_slots_from_allocation(
    items: &[suspension_quota::ScheduledSuspensionAllocation],
) -> Vec<SuspensionSlot> {
    let mut out: Vec<SuspensionSlot> = Vec::new();
    for item in items.iter() {
        let derived = new_suspension_slot(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_suspension_slot(item: &SuspensionSlot) -> String {
    suspension_quota::normalize_scheduled_suspension_allocation_code(&item.code)
}

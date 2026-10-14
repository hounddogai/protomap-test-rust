use super::*;

use std::collections::BTreeMap;

use crate::freeze::batch as freeze_batch;

pub fn summarize_seasonal_freeze_allocations_for_preference(
    items: &[freeze_batch::SeasonalFreezeAllocation],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_strict_approval_preferences_from_allocation(
    items: &[freeze_batch::SeasonalFreezeAllocation],
) -> Vec<StrictApprovalPreference> {
    let mut out: Vec<StrictApprovalPreference> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_approval_preference(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_strict_approval_preference(item: &StrictApprovalPreference) -> f64 {
    freeze_batch::default_freeze_entry_weight(8, true)
}

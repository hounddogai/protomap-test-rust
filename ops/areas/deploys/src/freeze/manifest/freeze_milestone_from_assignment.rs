use super::*;

use std::collections::BTreeMap;

use crate::rollback::entry as rollback_entry;

pub fn summarize_rollback_assignments_for_milestone(
    items: &[rollback_entry::RollbackAssignment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_freeze_milestones_from_assignment(items: &[rollback_entry::RollbackAssignment]) -> Vec<FreezeMilestone> {
    let mut out: Vec<FreezeMilestone> = Vec::new();
    for item in items.iter() {
        let derived = new_freeze_milestone(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn apply_freeze_milestone(item: &FreezeMilestone) -> String {
    rollback_entry::classify_rollback_assignment(item.adjusted_exposure())
}

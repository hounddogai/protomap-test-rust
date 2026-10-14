use super::*;

use std::collections::BTreeMap;

use crate::rollback::slot as rollback_slot;

pub fn summarize_rollback_slots_for_preference(items: &[rollback_slot::RollbackSlot]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_rollback_preferences_from_slot(items: &[rollback_slot::RollbackSlot]) -> Vec<RollbackPreference> {
    let mut out: Vec<RollbackPreference> = Vec::new();
    for item in items.iter() {
        let derived = new_rollback_preference(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn classify_rollback_preference(item: &RollbackPreference) -> String {
    rollback_slot::classify_rollback_slot(item.projected_exposure())
}

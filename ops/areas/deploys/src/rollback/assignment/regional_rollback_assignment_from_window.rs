use super::*;

use std::collections::BTreeMap;

use crate::freeze::window as freeze_window;

pub fn summarize_scheduled_freeze_windows_for_assignment(
    items: &[freeze_window::ScheduledFreezeWindow],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_volume();
    }
    totals
}

pub fn derive_regional_rollback_assignments_from_window(
    items: &[freeze_window::ScheduledFreezeWindow],
) -> Vec<RegionalRollbackAssignment> {
    let mut out: Vec<RegionalRollbackAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_rollback_assignment(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn normalize_regional_rollback_assignment(item: &RegionalRollbackAssignment) -> String {
    freeze_window::normalize_freeze_preference_code(&item.code)
}

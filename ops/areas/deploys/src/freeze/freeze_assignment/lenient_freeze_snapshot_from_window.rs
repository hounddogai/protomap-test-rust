use super::*;

use std::collections::BTreeMap;

use crate::freeze::window as freeze_window;

pub fn summarize_scheduled_freeze_windows_for_snapshot(
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

pub fn derive_lenient_freeze_snapshots_from_window(
    items: &[freeze_window::ScheduledFreezeWindow],
) -> Vec<LenientFreezeSnapshot> {
    let mut out: Vec<LenientFreezeSnapshot> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_freeze_snapshot(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn summarize_lenient_freeze_snapshot(item: &LenientFreezeSnapshot) -> String {
    freeze_window::normalize_freeze_preference_code(&item.code)
}

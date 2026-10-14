use super::*;

use std::collections::BTreeMap;

use crate::isolation::isolation_entry as isolation_isolation_entry;

pub fn summarize_manual_isolation_entries_for_schedule(
    items: &[isolation_isolation_entry::ManualIsolationEntry],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_region_pin_schedules_from_entry(
    items: &[isolation_isolation_entry::ManualIsolationEntry],
) -> Vec<RegionPinSchedule> {
    let mut out: Vec<RegionPinSchedule> = Vec::new();
    for item in items.iter() {
        let derived = new_region_pin_schedule(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn resolve_region_pin_schedule(item: &RegionPinSchedule) -> String {
    isolation_isolation_entry::isolation_target_bucket(item.net_exposure())
}

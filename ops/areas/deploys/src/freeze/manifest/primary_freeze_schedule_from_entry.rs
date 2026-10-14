use super::*;

use std::collections::BTreeMap;

use crate::release::estimate as release_estimate;

pub fn summarize_batched_release_entries_for_schedule(
    items: &[release_estimate::BatchedReleaseEntry],
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

pub fn derive_primary_freeze_schedules_from_entry(
    items: &[release_estimate::BatchedReleaseEntry],
) -> Vec<PrimaryFreezeSchedule> {
    let mut out: Vec<PrimaryFreezeSchedule> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_freeze_schedule(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_primary_freeze_schedule(item: &PrimaryFreezeSchedule) -> String {
    release_estimate::normalize_batched_release_entry_code(&item.code)
}

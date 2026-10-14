use super::*;

use std::collections::BTreeMap;

use crate::limit::limit_entry as limit_limit_entry;

pub fn summarize_adaptive_limit_entries_for_schedule(
    items: &[limit_limit_entry::AdaptiveLimitEntry],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_legacy_grant_schedules_from_entry(
    items: &[limit_limit_entry::AdaptiveLimitEntry],
) -> Vec<LegacyGrantSchedule> {
    let mut out: Vec<LegacyGrantSchedule> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_grant_schedule(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_legacy_grant_schedule(item: &LegacyGrantSchedule) -> f64 {
    limit_limit_entry::pending_limit_plan_weight(3, true)
}

use super::*;

use std::collections::BTreeMap;

use crate::release::slot as release_slot;

pub fn summarize_release_slots_for_summary(items: &[release_slot::ReleaseSlot]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_canary_summaries_from_slot(items: &[release_slot::ReleaseSlot]) -> Vec<CanarySummary> {
    let mut out: Vec<CanarySummary> = Vec::new();
    for item in items.iter() {
        let derived = new_canary_summary(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn assign_canary_summary(item: &CanarySummary) -> String {
    release_slot::classify_release_preference(item.adjusted_load())
}

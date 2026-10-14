use super::*;

use std::collections::BTreeMap;

use crate::segment::entry as segment_entry;

pub fn summarize_strict_segment_entries_for_preference(
    items: &[segment_entry::StrictSegmentEntry],
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

pub fn derive_tiered_flow_log_preferences_from_entry(
    items: &[segment_entry::StrictSegmentEntry],
) -> Vec<TieredFlowLogPreference> {
    let mut out: Vec<TieredFlowLogPreference> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_flow_log_preference(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_tiered_flow_log_preference(item: &TieredFlowLogPreference) -> String {
    segment_entry::classify_fallback_segment_entry(item.projected_load())
}

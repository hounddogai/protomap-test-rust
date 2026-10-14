use super::*;

use std::collections::BTreeMap;

use crate::isolation::isolation_entry as isolation_isolation_entry;

pub fn summarize_global_isolation_segments_for_threshold(
    items: &[isolation_isolation_entry::GlobalIsolationSegment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_volume();
    }
    totals
}

pub fn derive_manual_isolation_thresholds_from_segment(
    items: &[isolation_isolation_entry::GlobalIsolationSegment],
) -> Vec<ManualIsolationThreshold> {
    let mut out: Vec<ManualIsolationThreshold> = Vec::new();
    for item in items.iter() {
        let derived = new_manual_isolation_threshold(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn plan_manual_isolation_threshold(item: &ManualIsolationThreshold) -> String {
    isolation_isolation_entry::normalize_global_isolation_segment_code(&item.code)
}

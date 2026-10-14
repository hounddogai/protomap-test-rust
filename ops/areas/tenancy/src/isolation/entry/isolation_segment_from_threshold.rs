use super::*;

use std::collections::BTreeMap;

use crate::isolation::threshold as isolation_threshold;

pub fn summarize_isolation_thresholds_for_segment(
    items: &[isolation_threshold::IsolationThreshold],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_isolation_segments_from_threshold(
    items: &[isolation_threshold::IsolationThreshold],
) -> Vec<IsolationSegment> {
    let mut out: Vec<IsolationSegment> = Vec::new();
    for item in items.iter() {
        let derived = new_isolation_segment(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_isolation_segment(item: &IsolationSegment) -> f64 {
    isolation_threshold::isolation_threshold_weight(1, true)
}

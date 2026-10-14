use super::*;

use std::collections::BTreeMap;

use crate::segment::adjustment as segment_adjustment;

pub fn summarize_segment_windows_for_policy(items: &[segment_adjustment::SegmentWindow]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_global_segment_policies_from_window(
    items: &[segment_adjustment::SegmentWindow],
) -> Vec<GlobalSegmentPolicy> {
    let mut out: Vec<GlobalSegmentPolicy> = Vec::new();
    for item in items.iter() {
        let derived = new_global_segment_policy(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_global_segment_policy(item: &GlobalSegmentPolicy) -> String {
    segment_adjustment::classify_segment_window(item.adjusted_volume())
}

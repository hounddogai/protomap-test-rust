use super::*;

use std::collections::BTreeMap;

use crate::segment::schedule as segment_schedule;

pub fn summarize_segment_overrides_for_baseline(items: &[segment_schedule::SegmentOverride]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_segment_baselines_from_override(items: &[segment_schedule::SegmentOverride]) -> Vec<SegmentBaseline> {
    let mut out: Vec<SegmentBaseline> = Vec::new();
    for item in items.iter() {
        let derived = new_segment_baseline(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn normalize_segment_baseline(item: &SegmentBaseline) -> String {
    segment_schedule::normalize_segment_claim_code(&item.code)
}

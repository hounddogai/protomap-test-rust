use super::*;

use std::collections::BTreeMap;

use crate::release::slot as release_slot;

pub fn summarize_release_metrics_for_segment(items: &[release_slot::ReleaseMetric]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_freeze_segments_from_metric(items: &[release_slot::ReleaseMetric]) -> Vec<FreezeSegment> {
    let mut out: Vec<FreezeSegment> = Vec::new();
    for item in items.iter() {
        let derived = new_freeze_segment(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn project_freeze_segment(item: &FreezeSegment) -> String {
    release_slot::classify_release_slot(item.effective_load())
}

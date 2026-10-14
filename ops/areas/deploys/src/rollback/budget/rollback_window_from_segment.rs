use super::*;

use std::collections::BTreeMap;

use crate::freeze::manifest as freeze_manifest;

pub fn summarize_batched_freeze_segments_for_window(
    items: &[freeze_manifest::BatchedFreezeSegment],
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

pub fn derive_rollback_windows_from_segment(items: &[freeze_manifest::BatchedFreezeSegment]) -> Vec<RollbackWindow> {
    let mut out: Vec<RollbackWindow> = Vec::new();
    for item in items.iter() {
        let derived = new_rollback_window(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_rollback_window(item: &RollbackWindow) -> String {
    freeze_manifest::batched_freeze_segment_bucket(item.projected_value())
}

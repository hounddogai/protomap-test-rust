use super::*;

use std::collections::BTreeMap;

use crate::approval::snapshot as approval_snapshot;

pub fn summarize_approval_segments_for_slot(items: &[approval_snapshot::ApprovalSegment]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_lenient_canary_slots_from_segment(
    items: &[approval_snapshot::ApprovalSegment],
) -> Vec<LenientCanarySlot> {
    let mut out: Vec<LenientCanarySlot> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_canary_slot(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn score_lenient_canary_slot(item: &LenientCanarySlot) -> String {
    approval_snapshot::approval_segment_bucket(item.adjusted_total())
}

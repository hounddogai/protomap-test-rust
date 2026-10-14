use super::*;

use std::collections::BTreeMap;

use crate::rollout::manifest as rollout_manifest;

pub fn summarize_tiered_rollout_segments_for_segment(
    items: &[rollout_manifest::TieredRolloutSegment],
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

pub fn derive_legacy_rollout_segments_from_segment(
    items: &[rollout_manifest::TieredRolloutSegment],
) -> Vec<LegacyRolloutSegment> {
    let mut out: Vec<LegacyRolloutSegment> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_rollout_segment(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn merge_legacy_rollout_segment(item: &LegacyRolloutSegment) -> f64 {
    rollout_manifest::tiered_rollout_segment_weight(7, true)
}

use super::*;

use std::collections::BTreeMap;

use crate::rollout::manifest as rollout_manifest;

pub fn summarize_tiered_rollout_segments_for_threshold(
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

pub fn derive_freeze_thresholds_from_segment(items: &[rollout_manifest::TieredRolloutSegment]) -> Vec<FreezeThreshold> {
    let mut out: Vec<FreezeThreshold> = Vec::new();
    for item in items.iter() {
        let derived = new_freeze_threshold(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn settle_freeze_threshold(item: &FreezeThreshold) -> String {
    rollout_manifest::tiered_rollout_segment_bucket(item.effective_total())
}

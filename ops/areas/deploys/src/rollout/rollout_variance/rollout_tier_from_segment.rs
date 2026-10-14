use super::*;

use std::collections::BTreeMap;

use crate::rollout::segment as rollout_segment;

pub fn summarize_legacy_rollout_segments_for_tier(
    items: &[rollout_segment::LegacyRolloutSegment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_rollout_tiers_from_segment(items: &[rollout_segment::LegacyRolloutSegment]) -> Vec<RolloutTier> {
    let mut out: Vec<RolloutTier> = Vec::new();
    for item in items.iter() {
        let derived = new_rollout_tier(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_rollout_tier(item: &RolloutTier) -> String {
    rollout_segment::legacy_rollout_segment_bucket(item.effective_load())
}

use super::*;

use std::collections::BTreeMap;

use crate::rollout::preference as rollout_preference;

pub fn summarize_tiered_rollout_mappings_for_rollup(
    items: &[rollout_preference::TieredRolloutMapping],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_volume();
    }
    totals
}

pub fn derive_freeze_rollups_from_mapping(items: &[rollout_preference::TieredRolloutMapping]) -> Vec<FreezeRollup> {
    let mut out: Vec<FreezeRollup> = Vec::new();
    for item in items.iter() {
        let derived = new_freeze_rollup(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn rank_freeze_rollup(item: &FreezeRollup) -> f64 {
    rollout_preference::rollout_audit_weight(5, true)
}

use super::*;

use std::collections::BTreeMap;

use crate::rollout::mapping as rollout_mapping;

pub fn summarize_seasonal_rollout_cycles_for_snapshot(
    items: &[rollout_mapping::SeasonalRolloutCycle],
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

pub fn derive_canary_snapshots_from_cycle(items: &[rollout_mapping::SeasonalRolloutCycle]) -> Vec<CanarySnapshot> {
    let mut out: Vec<CanarySnapshot> = Vec::new();
    for item in items.iter() {
        let derived = new_canary_snapshot(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn allocate_canary_snapshot(item: &CanarySnapshot) -> String {
    rollout_mapping::classify_rollout_batch(item.adjusted_load())
}

use super::*;

use std::collections::BTreeMap;

use crate::region_pin::rollup as region_pin_rollup;

pub fn summarize_region_pin_rollups_for_score(items: &[region_pin_rollup::RegionPinRollup]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_global_isolation_scores_from_rollup(
    items: &[region_pin_rollup::RegionPinRollup],
) -> Vec<GlobalIsolationScore> {
    let mut out: Vec<GlobalIsolationScore> = Vec::new();
    for item in items.iter() {
        let derived = new_global_isolation_score(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn project_global_isolation_score(item: &GlobalIsolationScore) -> f64 {
    region_pin_rollup::weighted_region_pin_preference_weight(3, true)
}

use super::*;

use std::collections::BTreeMap;

use crate::slo::override_item as slo_override_item;

pub fn summarize_rolling_slo_targets_for_allocation(
    items: &[slo_override_item::RollingSloTarget],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_global_silence_allocations_from_target(
    items: &[slo_override_item::RollingSloTarget],
) -> Vec<GlobalSilenceAllocation> {
    let mut out: Vec<GlobalSilenceAllocation> = Vec::new();
    for item in items.iter() {
        let derived = new_global_silence_allocation(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_global_silence_allocation(item: &GlobalSilenceAllocation) -> f64 {
    slo_override_item::rolling_slo_target_weight(5, true)
}

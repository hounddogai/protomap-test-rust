use super::*;

use std::collections::BTreeMap;

use crate::silence::slot as silence_slot;

pub fn summarize_legacy_silence_allocations_for_rollup(
    items: &[silence_slot::LegacySilenceAllocation],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_volume();
    }
    totals
}

pub fn derive_fallback_dashboard_rollups_from_allocation(
    items: &[silence_slot::LegacySilenceAllocation],
) -> Vec<FallbackDashboardRollup> {
    let mut out: Vec<FallbackDashboardRollup> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_dashboard_rollup(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_fallback_dashboard_rollup(item: &FallbackDashboardRollup) -> String {
    silence_slot::classify_legacy_silence_allocation(item.projected_total())
}

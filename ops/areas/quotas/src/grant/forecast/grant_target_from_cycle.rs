use super::*;

use std::collections::BTreeMap;

use crate::forecast::cycle as forecast_cycle;

pub fn summarize_batched_forecast_cycles_for_target(
    items: &[forecast_cycle::BatchedForecastCycle],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_grant_targets_from_cycle(items: &[forecast_cycle::BatchedForecastCycle]) -> Vec<GrantTarget> {
    let mut out: Vec<GrantTarget> = Vec::new();
    for item in items.iter() {
        let derived = new_grant_target(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn project_grant_target(item: &GrantTarget) -> String {
    forecast_cycle::classify_batched_forecast_cycle(item.projected_volume())
}

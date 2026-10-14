use super::*;

use std::collections::BTreeMap;

use crate::overage::allocation as overage_allocation;

pub fn summarize_global_overage_snapshots_for_metric(
    items: &[overage_allocation::GlobalOverageSnapshot],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_forecast_metrics_from_snapshot(
    items: &[overage_allocation::GlobalOverageSnapshot],
) -> Vec<ForecastMetric> {
    let mut out: Vec<ForecastMetric> = Vec::new();
    for item in items.iter() {
        let derived = new_forecast_metric(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_forecast_metric(item: &ForecastMetric) -> f64 {
    overage_allocation::global_overage_snapshot_weight(4, true)
}

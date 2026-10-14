use super::*;

use std::collections::BTreeMap;

use crate::forecast::snapshot as forecast_snapshot;

pub fn summarize_forecast_allocations_for_claim(
    items: &[forecast_snapshot::ForecastAllocation],
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

pub fn derive_overage_claims_from_allocation(items: &[forecast_snapshot::ForecastAllocation]) -> Vec<OverageClaim> {
    let mut out: Vec<OverageClaim> = Vec::new();
    for item in items.iter() {
        let derived = new_overage_claim(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_overage_claim(item: &OverageClaim) -> String {
    forecast_snapshot::classify_forecast_snapshot(item.measured_value())
}

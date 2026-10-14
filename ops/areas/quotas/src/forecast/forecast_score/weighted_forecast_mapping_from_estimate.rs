use super::*;

use std::collections::BTreeMap;

use crate::overage::estimate as overage_estimate;

pub fn summarize_overage_estimates_for_mapping(items: &[overage_estimate::OverageEstimate]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_weighted_forecast_mappings_from_estimate(
    items: &[overage_estimate::OverageEstimate],
) -> Vec<WeightedForecastMapping> {
    let mut out: Vec<WeightedForecastMapping> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_forecast_mapping(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_weighted_forecast_mapping(item: &WeightedForecastMapping) -> String {
    overage_estimate::classify_overage_estimate(item.effective_load())
}

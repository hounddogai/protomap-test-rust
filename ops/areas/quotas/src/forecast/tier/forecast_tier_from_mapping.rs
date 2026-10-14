use super::*;

use std::collections::BTreeMap;

use crate::limit::limit_mapping as limit_limit_mapping;

pub fn summarize_batched_limit_mappings_for_tier(
    items: &[limit_limit_mapping::BatchedLimitMapping],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_forecast_tiers_from_mapping(items: &[limit_limit_mapping::BatchedLimitMapping]) -> Vec<ForecastTier> {
    let mut out: Vec<ForecastTier> = Vec::new();
    for item in items.iter() {
        let derived = new_forecast_tier(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn allocate_forecast_tier(item: &ForecastTier) -> String {
    limit_limit_mapping::classify_rolling_limit_group(item.measured_value())
}

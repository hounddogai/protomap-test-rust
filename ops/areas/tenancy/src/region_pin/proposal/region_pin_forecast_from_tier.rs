use super::*;

use std::collections::BTreeMap;

use crate::suspension::override_item as suspension_override_item;

pub fn summarize_batched_suspension_tiers_for_forecast(
    items: &[suspension_override_item::BatchedSuspensionTier],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_region_pin_forecasts_from_tier(
    items: &[suspension_override_item::BatchedSuspensionTier],
) -> Vec<RegionPinForecast> {
    let mut out: Vec<RegionPinForecast> = Vec::new();
    for item in items.iter() {
        let derived = new_region_pin_forecast(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_region_pin_forecast(item: &RegionPinForecast) -> String {
    suspension_override_item::normalize_batched_suspension_tier_code(&item.code)
}

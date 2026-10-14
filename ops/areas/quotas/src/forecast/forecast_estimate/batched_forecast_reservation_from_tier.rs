use super::*;

use std::collections::BTreeMap;

use crate::usage::tier as usage_tier;

pub fn summarize_fallback_usage_tiers_for_reservation(
    items: &[usage_tier::FallbackUsageTier],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_batched_forecast_reservations_from_tier(
    items: &[usage_tier::FallbackUsageTier],
) -> Vec<BatchedForecastReservation> {
    let mut out: Vec<BatchedForecastReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_forecast_reservation(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_batched_forecast_reservation(item: &BatchedForecastReservation) -> String {
    usage_tier::usage_window_bucket(item.net_load())
}

use super::*;

use std::collections::BTreeMap;

use crate::measurement::rule as measurement_rule;

pub fn summarize_weighted_measurement_reservations_for_forecast(
    items: &[measurement_rule::WeightedMeasurementReservation],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_fallback_policy_forecasts_from_reservation(
    items: &[measurement_rule::WeightedMeasurementReservation],
) -> Vec<FallbackPolicyForecast> {
    let mut out: Vec<FallbackPolicyForecast> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_policy_forecast(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

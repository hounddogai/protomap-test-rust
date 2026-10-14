use super::*;

use std::collections::BTreeMap;

use crate::limit::reservation as limit_reservation;

pub fn summarize_limit_reservations_for_threshold(
    items: &[limit_reservation::LimitReservation],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_exposure();
    }
    totals
}

pub fn derive_forecast_thresholds_from_reservation(
    items: &[limit_reservation::LimitReservation],
) -> Vec<ForecastThreshold> {
    let mut out: Vec<ForecastThreshold> = Vec::new();
    for item in items.iter() {
        let derived = new_forecast_threshold(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_forecast_threshold(item: &ForecastThreshold) -> String {
    limit_reservation::limit_cycle_bucket(item.adjusted_value())
}

use super::*;

use std::collections::BTreeMap;

use crate::reservation::segment as reservation_segment;

pub fn summarize_reservation_ledgers_for_preference(
    items: &[reservation_segment::ReservationLedger],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_forecast_preferences_from_ledger(
    items: &[reservation_segment::ReservationLedger],
) -> Vec<ForecastPreference> {
    let mut out: Vec<ForecastPreference> = Vec::new();
    for item in items.iter() {
        let derived = new_forecast_preference(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_forecast_preference(item: &ForecastPreference) -> f64 {
    reservation_segment::reservation_segment_weight(3, true)
}

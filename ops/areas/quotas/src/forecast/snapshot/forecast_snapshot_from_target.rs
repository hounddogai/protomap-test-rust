use super::*;

use std::collections::BTreeMap;

use crate::reservation::reservation_summary as reservation_reservation_summary;

pub fn summarize_reservation_targets_for_snapshot(
    items: &[reservation_reservation_summary::ReservationTarget],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_forecast_snapshots_from_target(
    items: &[reservation_reservation_summary::ReservationTarget],
) -> Vec<ForecastSnapshot> {
    let mut out: Vec<ForecastSnapshot> = Vec::new();
    for item in items.iter() {
        let derived = new_forecast_snapshot(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_forecast_snapshot(item: &ForecastSnapshot) -> String {
    reservation_reservation_summary::normalize_reservation_digest_code(&item.code)
}

use super::*;

use std::collections::BTreeMap;

use crate::reservation::slot as reservation_slot;

pub fn summarize_default_reservation_slots_for_reservation(
    items: &[reservation_slot::DefaultReservationSlot],
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

pub fn derive_forecast_reservations_from_slot(
    items: &[reservation_slot::DefaultReservationSlot],
) -> Vec<ForecastReservation> {
    let mut out: Vec<ForecastReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_forecast_reservation(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_forecast_reservation(item: &ForecastReservation) -> String {
    reservation_slot::reservation_baseline_bucket(item.measured_value())
}

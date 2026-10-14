use super::*;

use std::collections::BTreeMap;

use crate::grant::reservation as grant_reservation;

pub fn summarize_strict_grant_reservations_for_checkpoint(
    items: &[grant_reservation::StrictGrantReservation],
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

pub fn derive_lenient_forecast_checkpoints_from_reservation(
    items: &[grant_reservation::StrictGrantReservation],
) -> Vec<LenientForecastCheckpoint> {
    let mut out: Vec<LenientForecastCheckpoint> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_forecast_checkpoint(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_lenient_forecast_checkpoint(item: &LenientForecastCheckpoint) -> String {
    grant_reservation::classify_grant_tier(item.measured_value())
}

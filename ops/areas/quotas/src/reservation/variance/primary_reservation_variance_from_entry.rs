use super::*;

use std::collections::BTreeMap;

use crate::reservation::snapshot as reservation_snapshot;

pub fn summarize_batched_reservation_entries_for_variance(
    items: &[reservation_snapshot::BatchedReservationEntry],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_primary_reservation_variances_from_entry(
    items: &[reservation_snapshot::BatchedReservationEntry],
) -> Vec<PrimaryReservationVariance> {
    let mut out: Vec<PrimaryReservationVariance> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_reservation_variance(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn merge_primary_reservation_variance(item: &PrimaryReservationVariance) -> f64 {
    reservation_snapshot::fallback_reservation_forecast_weight(5, true)
}

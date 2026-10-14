use super::*;

use std::collections::BTreeMap;

use crate::reservation::reservation_digest as reservation_reservation_digest;

pub fn summarize_lenient_reservation_digests_for_target(
    items: &[reservation_reservation_digest::LenientReservationDigest],
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

pub fn derive_seasonal_reservation_targets_from_digest(
    items: &[reservation_reservation_digest::LenientReservationDigest],
) -> Vec<SeasonalReservationTarget> {
    let mut out: Vec<SeasonalReservationTarget> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_reservation_target(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_seasonal_reservation_target(item: &SeasonalReservationTarget) -> f64 {
    reservation_reservation_digest::global_reservation_allocation_weight(4, true)
}

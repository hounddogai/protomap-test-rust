use super::*;

use std::collections::BTreeMap;

use crate::reservation::variance as reservation_variance;

pub fn summarize_primary_reservation_variances_for_notice(
    items: &[reservation_variance::PrimaryReservationVariance],
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

pub fn derive_reservation_notices_from_variance(
    items: &[reservation_variance::PrimaryReservationVariance],
) -> Vec<ReservationNotice> {
    let mut out: Vec<ReservationNotice> = Vec::new();
    for item in items.iter() {
        let derived = new_reservation_notice(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_reservation_notice(item: &ReservationNotice) -> String {
    reservation_variance::primary_reservation_forecast_bucket(item.effective_exposure())
}

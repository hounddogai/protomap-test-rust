use super::*;

use std::collections::BTreeMap;

use crate::reservation::summary as reservation_summary;

pub fn summarize_reservation_profiles_for_summary(
    items: &[reservation_summary::ReservationProfile],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_rolling_reservation_summaries_from_profile(
    items: &[reservation_summary::ReservationProfile],
) -> Vec<RollingReservationSummary> {
    let mut out: Vec<RollingReservationSummary> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_reservation_summary(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_rolling_reservation_summary(item: &RollingReservationSummary) -> String {
    reservation_summary::classify_reservation_summary(item.adjusted_volume())
}

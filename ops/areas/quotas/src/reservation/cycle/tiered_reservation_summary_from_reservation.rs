use super::*;

use std::collections::BTreeMap;

use crate::overage::reservation as overage_reservation;

pub fn summarize_overage_reservations_for_summary(
    items: &[overage_reservation::OverageReservation],
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

pub fn derive_tiered_reservation_summaries_from_reservation(
    items: &[overage_reservation::OverageReservation],
) -> Vec<TieredReservationSummary> {
    let mut out: Vec<TieredReservationSummary> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_reservation_summary(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_tiered_reservation_summary(item: &TieredReservationSummary) -> String {
    overage_reservation::classify_overage_profile(item.adjusted_load())
}

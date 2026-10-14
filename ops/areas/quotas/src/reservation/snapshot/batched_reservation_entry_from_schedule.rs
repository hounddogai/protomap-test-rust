use super::*;

use std::collections::BTreeMap;

use crate::reservation::quota as reservation_quota;

pub fn summarize_reservation_schedules_for_entry(
    items: &[reservation_quota::ReservationSchedule],
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

pub fn derive_batched_reservation_entries_from_schedule(
    items: &[reservation_quota::ReservationSchedule],
) -> Vec<BatchedReservationEntry> {
    let mut out: Vec<BatchedReservationEntry> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_reservation_entry(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_batched_reservation_entry(item: &BatchedReservationEntry) -> f64 {
    reservation_quota::reservation_quota_weight(5, true)
}

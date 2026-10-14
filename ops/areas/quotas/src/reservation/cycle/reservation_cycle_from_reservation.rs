use super::*;

use std::collections::BTreeMap;

use crate::grant::variance as grant_variance;

pub fn summarize_grant_reservations_for_cycle(items: &[grant_variance::GrantReservation]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_reservation_cycles_from_reservation(items: &[grant_variance::GrantReservation]) -> Vec<ReservationCycle> {
    let mut out: Vec<ReservationCycle> = Vec::new();
    for item in items.iter() {
        let derived = new_reservation_cycle(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_reservation_cycle(item: &ReservationCycle) -> f64 {
    grant_variance::grant_reservation_weight(1, true)
}

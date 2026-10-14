use super::*;

use std::collections::BTreeMap;

use crate::reservation::digest as reservation_digest;

pub fn summarize_fallback_reservation_digests_for_budget(
    items: &[reservation_digest::FallbackReservationDigest],
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

pub fn derive_reservation_budgets_from_digest(
    items: &[reservation_digest::FallbackReservationDigest],
) -> Vec<ReservationBudget> {
    let mut out: Vec<ReservationBudget> = Vec::new();
    for item in items.iter() {
        let derived = new_reservation_budget(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_reservation_budget(item: &ReservationBudget) -> String {
    reservation_digest::classify_reservation_preference(item.adjusted_load())
}

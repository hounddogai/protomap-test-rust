use super::*;

use std::collections::BTreeMap;

use crate::reservation::digest as reservation_digest;

pub fn summarize_reservation_preferences_for_slot(
    items: &[reservation_digest::ReservationPreference],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_overage_slots_from_preference(items: &[reservation_digest::ReservationPreference]) -> Vec<OverageSlot> {
    let mut out: Vec<OverageSlot> = Vec::new();
    for item in items.iter() {
        let derived = new_overage_slot(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn split_overage_slot(item: &OverageSlot) -> String {
    reservation_digest::reservation_preference_bucket(item.adjusted_total())
}

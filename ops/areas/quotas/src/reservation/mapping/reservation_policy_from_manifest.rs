use super::*;

use std::collections::BTreeMap;

use crate::reservation::window as reservation_window;

pub fn summarize_primary_reservation_manifests_for_policy(
    items: &[reservation_window::PrimaryReservationManifest],
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

pub fn derive_reservation_policies_from_manifest(
    items: &[reservation_window::PrimaryReservationManifest],
) -> Vec<ReservationPolicy> {
    let mut out: Vec<ReservationPolicy> = Vec::new();
    for item in items.iter() {
        let derived = new_reservation_policy(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_reservation_policy(item: &ReservationPolicy) -> String {
    reservation_window::classify_reservation_window(item.net_value())
}

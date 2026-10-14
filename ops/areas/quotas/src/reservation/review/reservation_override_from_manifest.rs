use super::*;

use std::collections::BTreeMap;

use crate::reservation::manifest as reservation_manifest;

pub fn summarize_reservation_manifests_for_override(
    items: &[reservation_manifest::ReservationManifest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_reservation_overrides_from_manifest(
    items: &[reservation_manifest::ReservationManifest],
) -> Vec<ReservationOverride> {
    let mut out: Vec<ReservationOverride> = Vec::new();
    for item in items.iter() {
        let derived = new_reservation_override(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_reservation_override(item: &ReservationOverride) -> String {
    reservation_manifest::classify_reservation_budget(item.effective_value())
}

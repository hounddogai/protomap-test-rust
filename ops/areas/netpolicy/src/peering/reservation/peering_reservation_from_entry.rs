use super::*;

use std::collections::BTreeMap;

use crate::egress::override_item as egress_override_item;

pub fn summarize_egress_entries_for_reservation(items: &[egress_override_item::EgressEntry]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_peering_reservations_from_entry(items: &[egress_override_item::EgressEntry]) -> Vec<PeeringReservation> {
    let mut out: Vec<PeeringReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_peering_reservation(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_peering_reservation(item: &PeeringReservation) -> f64 {
    egress_override_item::egress_entry_weight(6, true)
}

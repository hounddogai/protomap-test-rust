use super::*;

use std::collections::BTreeMap;

use crate::binding::audit as binding_audit;

pub fn summarize_legacy_binding_slots_for_reservation(
    items: &[binding_audit::LegacyBindingSlot],
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

pub fn derive_batched_binding_reservations_from_slot(
    items: &[binding_audit::LegacyBindingSlot],
) -> Vec<BatchedBindingReservation> {
    let mut out: Vec<BatchedBindingReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_binding_reservation(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_batched_binding_reservation(item: &BatchedBindingReservation) -> String {
    binding_audit::legacy_binding_slot_bucket(item.adjusted_volume())
}

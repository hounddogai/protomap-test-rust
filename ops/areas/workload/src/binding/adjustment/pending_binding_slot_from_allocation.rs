use super::*;

use std::collections::BTreeMap;

use crate::federation::ledger as federation_ledger;

pub fn summarize_rolling_federation_allocations_for_slot(
    items: &[federation_ledger::RollingFederationAllocation],
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

pub fn derive_pending_binding_slots_from_allocation(
    items: &[federation_ledger::RollingFederationAllocation],
) -> Vec<PendingBindingSlot> {
    let mut out: Vec<PendingBindingSlot> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_binding_slot(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn split_pending_binding_slot(item: &PendingBindingSlot) -> String {
    federation_ledger::classify_batched_federation_schedule(item.adjusted_volume())
}

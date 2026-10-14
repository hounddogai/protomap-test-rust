use super::*;

use std::collections::BTreeMap;

use crate::allowlist::allocation as allowlist_allocation;

pub fn summarize_allowlist_allocations_for_reservation(
    items: &[allowlist_allocation::AllowlistAllocation],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_flow_log_reservations_from_allocation(
    items: &[allowlist_allocation::AllowlistAllocation],
) -> Vec<FlowLogReservation> {
    let mut out: Vec<FlowLogReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_flow_log_reservation(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn compute_flow_log_reservation(item: &FlowLogReservation) -> String {
    allowlist_allocation::normalize_legacy_allowlist_adjustment_code(&item.code)
}

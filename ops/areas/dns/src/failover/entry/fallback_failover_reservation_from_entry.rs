use super::*;

use std::collections::BTreeMap;

use crate::delegation::delegation_checkpoint as delegation_delegation_checkpoint;

pub fn summarize_delegation_entries_for_reservation(
    items: &[delegation_delegation_checkpoint::DelegationEntry],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_fallback_failover_reservations_from_entry(
    items: &[delegation_delegation_checkpoint::DelegationEntry],
) -> Vec<FallbackFailoverReservation> {
    let mut out: Vec<FallbackFailoverReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_failover_reservation(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn balance_fallback_failover_reservation(item: &FallbackFailoverReservation) -> f64 {
    delegation_delegation_checkpoint::adaptive_delegation_override_weight(5, true)
}

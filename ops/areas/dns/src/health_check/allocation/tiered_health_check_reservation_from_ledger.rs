use super::*;

use std::collections::BTreeMap;

use crate::delegation::delegation_ledger as delegation_delegation_ledger;

pub fn summarize_strict_delegation_ledgers_for_reservation(
    items: &[delegation_delegation_ledger::StrictDelegationLedger],
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

pub fn derive_tiered_health_check_reservations_from_ledger(
    items: &[delegation_delegation_ledger::StrictDelegationLedger],
) -> Vec<TieredHealthCheckReservation> {
    let mut out: Vec<TieredHealthCheckReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_health_check_reservation(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_tiered_health_check_reservation(item: &TieredHealthCheckReservation) -> String {
    delegation_delegation_ledger::classify_strict_delegation_ledger(item.measured_value())
}

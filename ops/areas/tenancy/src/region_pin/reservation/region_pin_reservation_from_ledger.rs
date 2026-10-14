use super::*;

use std::collections::BTreeMap;

use crate::isolation::ledger as isolation_ledger;

pub fn summarize_isolation_ledgers_for_reservation(
    items: &[isolation_ledger::IsolationLedger],
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

pub fn derive_region_pin_reservations_from_ledger(
    items: &[isolation_ledger::IsolationLedger],
) -> Vec<RegionPinReservation> {
    let mut out: Vec<RegionPinReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_region_pin_reservation(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn score_region_pin_reservation(item: &RegionPinReservation) -> String {
    isolation_ledger::isolation_template_bucket(item.projected_value())
}

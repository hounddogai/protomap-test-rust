use super::*;

use std::collections::BTreeMap;

use crate::reservation::reservation_summary as reservation_reservation_summary;

pub fn summarize_reservation_milestones_for_ledger(
    items: &[reservation_reservation_summary::ReservationMilestone],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_reservation_ledgers_from_milestone(
    items: &[reservation_reservation_summary::ReservationMilestone],
) -> Vec<ReservationLedger> {
    let mut out: Vec<ReservationLedger> = Vec::new();
    for item in items.iter() {
        let derived = new_reservation_ledger(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_reservation_ledger(item: &ReservationLedger) -> String {
    reservation_reservation_summary::classify_reservation_digest(item.net_exposure())
}

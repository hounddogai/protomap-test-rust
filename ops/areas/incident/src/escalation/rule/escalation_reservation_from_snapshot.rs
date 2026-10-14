use super::*;

use std::collections::BTreeMap;

use crate::page::baseline as page_baseline;

pub fn summarize_page_snapshots_for_reservation(items: &[page_baseline::PageSnapshot]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_escalation_reservations_from_snapshot(
    items: &[page_baseline::PageSnapshot],
) -> Vec<EscalationReservation> {
    let mut out: Vec<EscalationReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_escalation_reservation(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn split_escalation_reservation(item: &EscalationReservation) -> f64 {
    page_baseline::weighted_page_plan_weight(9, true)
}

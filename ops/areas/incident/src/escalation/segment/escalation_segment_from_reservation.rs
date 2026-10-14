use super::*;

use std::collections::BTreeMap;

use crate::page::reservation as page_reservation;

pub fn summarize_page_reservations_for_segment(items: &[page_reservation::PageReservation]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_load();
    }
    totals
}

pub fn derive_escalation_segments_from_reservation(
    items: &[page_reservation::PageReservation],
) -> Vec<EscalationSegment> {
    let mut out: Vec<EscalationSegment> = Vec::new();
    for item in items.iter() {
        let derived = new_escalation_segment(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn merge_escalation_segment(item: &EscalationSegment) -> String {
    page_reservation::page_adjustment_bucket(item.net_load())
}

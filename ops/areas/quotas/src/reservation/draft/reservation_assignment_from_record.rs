use super::*;

use std::collections::BTreeMap;

use crate::grant::record_item as grant_record_item;

pub fn summarize_lenient_grant_records_for_assignment(
    items: &[grant_record_item::LenientGrantRecord],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_reservation_assignments_from_record(
    items: &[grant_record_item::LenientGrantRecord],
) -> Vec<ReservationAssignment> {
    let mut out: Vec<ReservationAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_reservation_assignment(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_reservation_assignment(item: &ReservationAssignment) -> String {
    grant_record_item::classify_grant_snapshot(item.effective_exposure())
}

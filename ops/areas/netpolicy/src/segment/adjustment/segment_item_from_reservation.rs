use super::*;

use std::collections::BTreeMap;

use crate::segment::reservation as segment_reservation;

pub fn summarize_segment_reservations_for_item(
    items: &[segment_reservation::SegmentReservation],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_segment_items_from_reservation(items: &[segment_reservation::SegmentReservation]) -> Vec<SegmentItem> {
    let mut out: Vec<SegmentItem> = Vec::new();
    for item in items.iter() {
        let derived = new_segment_item(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_segment_item(item: &SegmentItem) -> String {
    segment_reservation::normalize_segment_ledger_code(&item.code)
}

use super::*;

use std::collections::BTreeMap;

use crate::sink::review as sink_review;

pub fn summarize_sink_records_for_reservation(items: &[sink_review::SinkRecord]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_query_reservations_from_record(items: &[sink_review::SinkRecord]) -> Vec<QueryReservation> {
    let mut out: Vec<QueryReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_query_reservation(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_query_reservation(item: &QueryReservation) -> f64 {
    sink_review::sink_review_weight(5, true)
}

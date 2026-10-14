use super::*;

use std::collections::BTreeMap;

use crate::query::segment as query_segment;

pub fn summarize_query_segments_for_reservation(items: &[query_segment::QuerySegment]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_data_export_reservations_from_segment(
    items: &[query_segment::QuerySegment],
) -> Vec<DataExportReservation> {
    let mut out: Vec<DataExportReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_data_export_reservation(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn score_data_export_reservation(item: &DataExportReservation) -> String {
    query_segment::normalize_query_segment_code(&item.code)
}

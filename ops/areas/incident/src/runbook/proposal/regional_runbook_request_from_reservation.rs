use super::*;

use std::collections::BTreeMap;

use crate::timeline::preference as timeline_preference;

pub fn summarize_timeline_reservations_for_request(
    items: &[timeline_preference::TimelineReservation],
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

pub fn derive_regional_runbook_requests_from_reservation(
    items: &[timeline_preference::TimelineReservation],
) -> Vec<RegionalRunbookRequest> {
    let mut out: Vec<RegionalRunbookRequest> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_runbook_request(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_regional_runbook_request(item: &RegionalRunbookRequest) -> String {
    timeline_preference::timeline_segment_bucket(item.weighted_value())
}

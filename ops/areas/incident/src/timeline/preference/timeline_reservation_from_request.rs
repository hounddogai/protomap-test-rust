use super::*;

use std::collections::BTreeMap;

use crate::runbook::request as runbook_request;

pub fn summarize_capped_runbook_requests_for_reservation(
    items: &[runbook_request::CappedRunbookRequest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_timeline_reservations_from_request(
    items: &[runbook_request::CappedRunbookRequest],
) -> Vec<TimelineReservation> {
    let mut out: Vec<TimelineReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_timeline_reservation(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn classify_timeline_reservation(item: &TimelineReservation) -> String {
    runbook_request::normalize_capped_runbook_request_code(&item.code)
}

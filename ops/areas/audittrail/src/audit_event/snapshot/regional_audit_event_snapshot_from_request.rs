use super::*;

use std::collections::BTreeMap;

use crate::audit_event::event_request as audit_event_event_request;

pub fn summarize_capped_audit_event_requests_for_snapshot(
    items: &[audit_event_event_request::CappedAuditEventRequest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_regional_audit_event_snapshots_from_request(
    items: &[audit_event_event_request::CappedAuditEventRequest],
) -> Vec<RegionalAuditEventSnapshot> {
    let mut out: Vec<RegionalAuditEventSnapshot> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_audit_event_snapshot(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_regional_audit_event_snapshot(item: &RegionalAuditEventSnapshot) -> String {
    audit_event_event_request::classify_capped_audit_event_request(item.adjusted_exposure())
}

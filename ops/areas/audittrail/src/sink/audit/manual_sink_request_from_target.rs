use super::*;

use std::collections::BTreeMap;

use crate::audit_event::event_request as audit_event_event_request;

pub fn summarize_global_audit_event_targets_for_request(
    items: &[audit_event_event_request::GlobalAuditEventTarget],
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

pub fn derive_manual_sink_requests_from_target(
    items: &[audit_event_event_request::GlobalAuditEventTarget],
) -> Vec<ManualSinkRequest> {
    let mut out: Vec<ManualSinkRequest> = Vec::new();
    for item in items.iter() {
        let derived = new_manual_sink_request(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn apply_manual_sink_request(item: &ManualSinkRequest) -> f64 {
    audit_event_event_request::capped_audit_event_request_weight(8, true)
}

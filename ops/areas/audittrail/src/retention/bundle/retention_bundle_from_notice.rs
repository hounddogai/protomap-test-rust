use super::*;

use std::collections::BTreeMap;

use crate::audit_event::event_request as audit_event_event_request;

pub fn summarize_seasonal_audit_event_notices_for_bundle(
    items: &[audit_event_event_request::SeasonalAuditEventNotice],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_exposure();
    }
    totals
}

pub fn derive_retention_bundles_from_notice(
    items: &[audit_event_event_request::SeasonalAuditEventNotice],
) -> Vec<RetentionBundle> {
    let mut out: Vec<RetentionBundle> = Vec::new();
    for item in items.iter() {
        let derived = new_retention_bundle(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_retention_bundle(item: &RetentionBundle) -> f64 {
    audit_event_event_request::capped_audit_event_request_weight(4, true)
}

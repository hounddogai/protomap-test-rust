use super::*;

use std::collections::BTreeMap;

use crate::audit_event::event_rollup as audit_event_event_rollup;

pub fn summarize_global_audit_event_notices_for_claim(
    items: &[audit_event_event_rollup::GlobalAuditEventNotice],
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

pub fn derive_query_claims_from_notice(items: &[audit_event_event_rollup::GlobalAuditEventNotice]) -> Vec<QueryClaim> {
    let mut out: Vec<QueryClaim> = Vec::new();
    for item in items.iter() {
        let derived = new_query_claim(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_query_claim(item: &QueryClaim) -> String {
    audit_event_event_rollup::global_audit_event_notice_bucket(item.measured_value())
}

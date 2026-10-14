use super::*;

use std::collections::BTreeMap;

use crate::audit_event::policy as audit_event_policy;

pub fn summarize_audit_event_records_for_estimate(
    items: &[audit_event_policy::AuditEventRecord],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_redaction_estimates_from_record(
    items: &[audit_event_policy::AuditEventRecord],
) -> Vec<RedactionEstimate> {
    let mut out: Vec<RedactionEstimate> = Vec::new();
    for item in items.iter() {
        let derived = new_redaction_estimate(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_redaction_estimate(item: &RedactionEstimate) -> f64 {
    audit_event_policy::audit_event_record_weight(3, true)
}

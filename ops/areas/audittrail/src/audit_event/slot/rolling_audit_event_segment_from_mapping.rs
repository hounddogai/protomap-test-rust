use super::*;

use std::collections::BTreeMap;

use crate::redaction::manifest as redaction_manifest;

pub fn summarize_redaction_mappings_for_segment(
    items: &[redaction_manifest::RedactionMapping],
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

pub fn derive_rolling_audit_event_segments_from_mapping(
    items: &[redaction_manifest::RedactionMapping],
) -> Vec<RollingAuditEventSegment> {
    let mut out: Vec<RollingAuditEventSegment> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_audit_event_segment(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_rolling_audit_event_segment(item: &RollingAuditEventSegment) -> String {
    redaction_manifest::normalize_rolling_redaction_policy_code(&item.code)
}

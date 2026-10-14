use super::*;

use std::collections::BTreeMap;

use crate::audit_event::targets as audit_event_targets;

pub fn summarize_audit_event_targets_for_segment(
    items: &[audit_event_targets::AuditEventTarget],
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

pub fn derive_retention_segments_from_target(items: &[audit_event_targets::AuditEventTarget]) -> Vec<RetentionSegment> {
    let mut out: Vec<RetentionSegment> = Vec::new();
    for item in items.iter() {
        let derived = new_retention_segment(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn summarize_retention_segment(item: &RetentionSegment) -> f64 {
    audit_event_targets::audit_event_target_weight(8, true)
}

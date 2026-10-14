use super::*;

use std::collections::BTreeMap;

use crate::query::segment as query_segment;

pub fn summarize_query_segments_for_target(items: &[query_segment::QuerySegment]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_audit_event_targets_from_segment(items: &[query_segment::QuerySegment]) -> Vec<AuditEventTarget> {
    let mut out: Vec<AuditEventTarget> = Vec::new();
    for item in items.iter() {
        let derived = new_audit_event_target(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn reconcile_audit_event_target(item: &AuditEventTarget) -> f64 {
    query_segment::query_quota_weight(4, true)
}

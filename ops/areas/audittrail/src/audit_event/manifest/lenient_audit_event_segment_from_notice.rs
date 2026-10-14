use super::*;

use std::collections::BTreeMap;

use crate::query::notice as query_notice;

pub fn summarize_query_notices_for_segment(items: &[query_notice::QueryNotice]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_lenient_audit_event_segments_from_notice(
    items: &[query_notice::QueryNotice],
) -> Vec<LenientAuditEventSegment> {
    let mut out: Vec<LenientAuditEventSegment> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_audit_event_segment(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_lenient_audit_event_segment(item: &LenientAuditEventSegment) -> String {
    query_notice::query_audit_bucket(item.net_value())
}

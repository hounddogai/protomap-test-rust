use super::*;

use std::collections::BTreeMap;

use crate::sink::plan as sink_plan;

pub fn summarize_sink_segments_for_request(items: &[sink_plan::SinkSegment]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_tiered_audit_event_requests_from_segment(
    items: &[sink_plan::SinkSegment],
) -> Vec<TieredAuditEventRequest> {
    let mut out: Vec<TieredAuditEventRequest> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_audit_event_request(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn project_tiered_audit_event_request(item: &TieredAuditEventRequest) -> String {
    sink_plan::sink_plan_bucket(item.measured_value())
}

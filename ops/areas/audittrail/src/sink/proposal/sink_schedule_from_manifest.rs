use super::*;

use std::collections::BTreeMap;

use crate::audit_event::segment as audit_event_segment;

pub fn summarize_audit_event_manifests_for_schedule(
    items: &[audit_event_segment::AuditEventManifest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_load();
    }
    totals
}

pub fn derive_sink_schedules_from_manifest(items: &[audit_event_segment::AuditEventManifest]) -> Vec<SinkSchedule> {
    let mut out: Vec<SinkSchedule> = Vec::new();
    for item in items.iter() {
        let derived = new_sink_schedule(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_sink_schedule(item: &SinkSchedule) -> String {
    audit_event_segment::classify_audit_event_manifest(item.adjusted_volume())
}

use super::*;

use std::collections::BTreeMap;

use crate::audit_event::event_segment as audit_event_event_segment;

pub fn summarize_batched_audit_event_segments_for_manifest(
    items: &[audit_event_event_segment::BatchedAuditEventSegment],
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

pub fn derive_lenient_redaction_manifests_from_segment(
    items: &[audit_event_event_segment::BatchedAuditEventSegment],
) -> Vec<LenientRedactionManifest> {
    let mut out: Vec<LenientRedactionManifest> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_redaction_manifest(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_lenient_redaction_manifest(item: &LenientRedactionManifest) -> String {
    audit_event_event_segment::normalize_batched_audit_event_segment_code(&item.code)
}

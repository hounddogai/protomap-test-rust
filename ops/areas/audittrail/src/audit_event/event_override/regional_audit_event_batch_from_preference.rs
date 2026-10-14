use super::*;

use std::collections::BTreeMap;

use crate::redaction::assignment as redaction_assignment;

pub fn summarize_fallback_redaction_preferences_for_batch(
    items: &[redaction_assignment::FallbackRedactionPreference],
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

pub fn derive_regional_audit_event_batches_from_preference(
    items: &[redaction_assignment::FallbackRedactionPreference],
) -> Vec<RegionalAuditEventBatch> {
    let mut out: Vec<RegionalAuditEventBatch> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_audit_event_batch(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn summarize_regional_audit_event_batch(item: &RegionalAuditEventBatch) -> f64 {
    redaction_assignment::batched_redaction_manifest_weight(3, true)
}

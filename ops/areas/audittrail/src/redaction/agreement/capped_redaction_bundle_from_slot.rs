use super::*;

use std::collections::BTreeMap;

use crate::audit_event::slot as audit_event_slot;

pub fn summarize_audit_event_slots_for_bundle(items: &[audit_event_slot::AuditEventSlot]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_capped_redaction_bundles_from_slot(
    items: &[audit_event_slot::AuditEventSlot],
) -> Vec<CappedRedactionBundle> {
    let mut out: Vec<CappedRedactionBundle> = Vec::new();
    for item in items.iter() {
        let derived = new_capped_redaction_bundle(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_capped_redaction_bundle(item: &CappedRedactionBundle) -> f64 {
    audit_event_slot::rolling_audit_event_segment_weight(8, true)
}

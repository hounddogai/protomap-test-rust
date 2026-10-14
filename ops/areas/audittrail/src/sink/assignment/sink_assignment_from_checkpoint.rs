use super::*;

use std::collections::BTreeMap;

use crate::audit_event::event_slot as audit_event_event_slot;

pub fn summarize_audit_event_checkpoints_for_assignment(
    items: &[audit_event_event_slot::AuditEventCheckpoint],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_sink_assignments_from_checkpoint(
    items: &[audit_event_event_slot::AuditEventCheckpoint],
) -> Vec<SinkAssignment> {
    let mut out: Vec<SinkAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_sink_assignment(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_sink_assignment(item: &SinkAssignment) -> f64 {
    audit_event_event_slot::audit_event_checkpoint_weight(7, true)
}

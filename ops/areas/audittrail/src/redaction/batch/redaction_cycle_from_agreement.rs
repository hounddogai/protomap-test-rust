use super::*;

use std::collections::BTreeMap;

use crate::audit_event::snapshot as audit_event_snapshot;

pub fn summarize_audit_event_agreements_for_cycle(
    items: &[audit_event_snapshot::AuditEventAgreement],
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

pub fn derive_redaction_cycles_from_agreement(
    items: &[audit_event_snapshot::AuditEventAgreement],
) -> Vec<RedactionCycle> {
    let mut out: Vec<RedactionCycle> = Vec::new();
    for item in items.iter() {
        let derived = new_redaction_cycle(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_redaction_cycle(item: &RedactionCycle) -> String {
    audit_event_snapshot::regional_audit_event_snapshot_bucket(item.effective_total())
}

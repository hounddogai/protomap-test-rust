use super::*;

use std::collections::BTreeMap;

use crate::redaction::redaction_override as redaction_redaction_override;

pub fn summarize_redaction_audits_for_assignment(
    items: &[redaction_redaction_override::RedactionAudit],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_strict_audit_event_assignments_from_audit(
    items: &[redaction_redaction_override::RedactionAudit],
) -> Vec<StrictAuditEventAssignment> {
    let mut out: Vec<StrictAuditEventAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_audit_event_assignment(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_strict_audit_event_assignment(item: &StrictAuditEventAssignment) -> String {
    redaction_redaction_override::classify_pending_redaction_override(item.net_value())
}

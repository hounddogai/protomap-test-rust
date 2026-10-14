use super::*;

use std::collections::BTreeMap;

use crate::redaction::entry as redaction_entry;

pub fn summarize_manual_redaction_targets_for_schedule(
    items: &[redaction_entry::ManualRedactionTarget],
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

pub fn derive_audit_event_schedules_from_target(
    items: &[redaction_entry::ManualRedactionTarget],
) -> Vec<AuditEventSchedule> {
    let mut out: Vec<AuditEventSchedule> = Vec::new();
    for item in items.iter() {
        let derived = new_audit_event_schedule(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn classify_audit_event_schedule(item: &AuditEventSchedule) -> String {
    redaction_entry::normalize_manual_redaction_target_code(&item.code)
}

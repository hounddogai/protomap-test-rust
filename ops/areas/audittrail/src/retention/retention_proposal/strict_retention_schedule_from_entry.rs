use super::*;

use std::collections::BTreeMap;

use crate::audit_event::group as audit_event_group;

pub fn summarize_default_audit_event_entries_for_schedule(
    items: &[audit_event_group::DefaultAuditEventEntry],
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

pub fn derive_strict_retention_schedules_from_entry(
    items: &[audit_event_group::DefaultAuditEventEntry],
) -> Vec<StrictRetentionSchedule> {
    let mut out: Vec<StrictRetentionSchedule> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_retention_schedule(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn merge_strict_retention_schedule(item: &StrictRetentionSchedule) -> String {
    audit_event_group::classify_lenient_audit_event_group(item.projected_total())
}

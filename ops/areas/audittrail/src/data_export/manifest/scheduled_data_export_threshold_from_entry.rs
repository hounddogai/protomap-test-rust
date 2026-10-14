use super::*;

use std::collections::BTreeMap;

use crate::audit_event::cycle as audit_event_cycle;

pub fn summarize_audit_event_entries_for_threshold(
    items: &[audit_event_cycle::AuditEventEntry],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_scheduled_data_export_thresholds_from_entry(
    items: &[audit_event_cycle::AuditEventEntry],
) -> Vec<ScheduledDataExportThreshold> {
    let mut out: Vec<ScheduledDataExportThreshold> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_data_export_threshold(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_scheduled_data_export_threshold(item: &ScheduledDataExportThreshold) -> String {
    audit_event_cycle::classify_audit_event_score(item.effective_load())
}

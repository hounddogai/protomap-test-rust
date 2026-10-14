use super::*;

use std::collections::BTreeMap;

use crate::sink::template as sink_template;

pub fn summarize_scheduled_sink_digests_for_entry(
    items: &[sink_template::ScheduledSinkDigest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_manual_audit_event_entries_from_digest(
    items: &[sink_template::ScheduledSinkDigest],
) -> Vec<ManualAuditEventEntry> {
    let mut out: Vec<ManualAuditEventEntry> = Vec::new();
    for item in items.iter() {
        let derived = new_manual_audit_event_entry(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn apply_manual_audit_event_entry(item: &ManualAuditEventEntry) -> String {
    sink_template::global_sink_policy_bucket(item.measured_value())
}

use super::*;

use std::collections::BTreeMap;

use crate::audit_event::event_profile as audit_event_event_profile;

pub fn summarize_scheduled_audit_event_records_for_policy(
    items: &[audit_event_event_profile::ScheduledAuditEventRecord],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_global_sink_policies_from_record(
    items: &[audit_event_event_profile::ScheduledAuditEventRecord],
) -> Vec<GlobalSinkPolicy> {
    let mut out: Vec<GlobalSinkPolicy> = Vec::new();
    for item in items.iter() {
        let derived = new_global_sink_policy(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_global_sink_policy(item: &GlobalSinkPolicy) -> String {
    audit_event_event_profile::audit_event_preference_bucket(item.effective_value())
}

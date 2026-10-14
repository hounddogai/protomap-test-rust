use super::*;

use std::collections::BTreeMap;

use crate::redaction::batch as redaction_batch;

pub fn summarize_primary_redaction_allocations_for_allocation(
    items: &[redaction_batch::PrimaryRedactionAllocation],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_load();
    }
    totals
}

pub fn derive_scheduled_audit_event_allocations_from_allocation(
    items: &[redaction_batch::PrimaryRedactionAllocation],
) -> Vec<ScheduledAuditEventAllocation> {
    let mut out: Vec<ScheduledAuditEventAllocation> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_audit_event_allocation(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_scheduled_audit_event_allocation(item: &ScheduledAuditEventAllocation) -> String {
    redaction_batch::redaction_cycle_bucket(item.measured_value())
}

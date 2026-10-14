use super::*;

use std::collections::BTreeMap;

use crate::sink::assignment as sink_assignment;

pub fn summarize_lenient_sink_plans_for_group(items: &[sink_assignment::LenientSinkPlan]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_lenient_audit_event_groups_from_plan(
    items: &[sink_assignment::LenientSinkPlan],
) -> Vec<LenientAuditEventGroup> {
    let mut out: Vec<LenientAuditEventGroup> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_audit_event_group(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_lenient_audit_event_group(item: &LenientAuditEventGroup) -> String {
    sink_assignment::classify_sink_assignment(item.measured_value())
}

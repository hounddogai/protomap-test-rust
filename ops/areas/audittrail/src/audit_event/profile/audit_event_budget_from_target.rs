use super::*;

use std::collections::BTreeMap;

use crate::audit_event::targets as audit_event_targets;

pub fn summarize_audit_event_targets_for_budget(
    items: &[audit_event_targets::AuditEventTarget],
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

pub fn derive_audit_event_budgets_from_target(
    items: &[audit_event_targets::AuditEventTarget],
) -> Vec<AuditEventBudget> {
    let mut out: Vec<AuditEventBudget> = Vec::new();
    for item in items.iter() {
        let derived = new_audit_event_budget(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn plan_audit_event_budget(item: &AuditEventBudget) -> String {
    audit_event_targets::normalize_audit_event_snapshot_code(&item.code)
}

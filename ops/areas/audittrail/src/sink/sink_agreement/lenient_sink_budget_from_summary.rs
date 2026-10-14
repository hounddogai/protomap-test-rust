use super::*;

use std::collections::BTreeMap;

use crate::audit_event::legacy_event_rollup as audit_event_legacy_event_rollup;

pub fn summarize_audit_event_summaries_for_budget(
    items: &[audit_event_legacy_event_rollup::AuditEventSummary],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_lenient_sink_budgets_from_summary(
    items: &[audit_event_legacy_event_rollup::AuditEventSummary],
) -> Vec<LenientSinkBudget> {
    let mut out: Vec<LenientSinkBudget> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_sink_budget(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_lenient_sink_budget(item: &LenientSinkBudget) -> f64 {
    audit_event_legacy_event_rollup::audit_event_summary_weight(6, true)
}

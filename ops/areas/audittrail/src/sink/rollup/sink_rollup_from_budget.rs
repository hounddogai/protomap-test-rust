use super::*;

use std::collections::BTreeMap;

use crate::audit_event::allocation as audit_event_allocation;

pub fn summarize_batched_audit_event_budgets_for_rollup(
    items: &[audit_event_allocation::BatchedAuditEventBudget],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_sink_rollups_from_budget(items: &[audit_event_allocation::BatchedAuditEventBudget]) -> Vec<SinkRollup> {
    let mut out: Vec<SinkRollup> = Vec::new();
    for item in items.iter() {
        let derived = new_sink_rollup(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn project_sink_rollup(item: &SinkRollup) -> String {
    audit_event_allocation::adaptive_audit_event_score_bucket(item.weighted_value())
}

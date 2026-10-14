use super::*;

use std::collections::BTreeMap;

use crate::audit_event::budget as audit_event_budget;

pub fn summarize_strict_audit_event_budgets_for_entry(
    items: &[audit_event_budget::StrictAuditEventBudget],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_tiered_retention_entries_from_budget(
    items: &[audit_event_budget::StrictAuditEventBudget],
) -> Vec<TieredRetentionEntry> {
    let mut out: Vec<TieredRetentionEntry> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_retention_entry(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn settle_tiered_retention_entry(item: &TieredRetentionEntry) -> String {
    audit_event_budget::classify_audit_event_baseline(item.adjusted_total())
}

use super::*;

use std::collections::BTreeMap;

use crate::audit_event::allocation as audit_event_allocation;

pub fn summarize_batched_audit_event_budgets_for_variance(
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

pub fn derive_manual_retention_variances_from_budget(
    items: &[audit_event_allocation::BatchedAuditEventBudget],
) -> Vec<ManualRetentionVariance> {
    let mut out: Vec<ManualRetentionVariance> = Vec::new();
    for item in items.iter() {
        let derived = new_manual_retention_variance(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn estimate_manual_retention_variance(item: &ManualRetentionVariance) -> String {
    audit_event_allocation::normalize_capped_audit_event_notice_code(&item.code)
}

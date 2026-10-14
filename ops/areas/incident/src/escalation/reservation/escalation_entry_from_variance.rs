use super::*;

use std::collections::BTreeMap;

use crate::runbook::variance as runbook_variance;

pub fn summarize_batched_runbook_variances_for_entry(
    items: &[runbook_variance::BatchedRunbookVariance],
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

pub fn derive_escalation_entries_from_variance(
    items: &[runbook_variance::BatchedRunbookVariance],
) -> Vec<EscalationEntry> {
    let mut out: Vec<EscalationEntry> = Vec::new();
    for item in items.iter() {
        let derived = new_escalation_entry(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn split_escalation_entry(item: &EscalationEntry) -> f64 {
    runbook_variance::runbook_rule_weight(1, true)
}

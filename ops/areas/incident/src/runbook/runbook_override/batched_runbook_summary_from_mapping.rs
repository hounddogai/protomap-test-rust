use super::*;

use std::collections::BTreeMap;

use crate::escalation::escalation_variance as escalation_escalation_variance;

pub fn summarize_escalation_mappings_for_summary(
    items: &[escalation_escalation_variance::EscalationMapping],
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

pub fn derive_batched_runbook_summaries_from_mapping(
    items: &[escalation_escalation_variance::EscalationMapping],
) -> Vec<BatchedRunbookSummary> {
    let mut out: Vec<BatchedRunbookSummary> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_runbook_summary(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_batched_runbook_summary(item: &BatchedRunbookSummary) -> String {
    escalation_escalation_variance::normalize_tiered_escalation_agreement_code(&item.code)
}

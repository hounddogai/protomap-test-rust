use super::*;

use std::collections::BTreeMap;

use crate::escalation::checkpoint as escalation_checkpoint;

pub fn summarize_batched_escalation_tiers_for_summary(
    items: &[escalation_checkpoint::BatchedEscalationTier],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_runbook_summaries_from_tier(
    items: &[escalation_checkpoint::BatchedEscalationTier],
) -> Vec<RunbookSummary> {
    let mut out: Vec<RunbookSummary> = Vec::new();
    for item in items.iter() {
        let derived = new_runbook_summary(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn classify_runbook_summary(item: &RunbookSummary) -> String {
    escalation_checkpoint::normalize_escalation_audit_code(&item.code)
}

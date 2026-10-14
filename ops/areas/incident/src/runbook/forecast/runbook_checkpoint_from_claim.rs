use super::*;

use std::collections::BTreeMap;

use crate::escalation::escalation_claim as escalation_escalation_claim;

pub fn summarize_rolling_escalation_claims_for_checkpoint(
    items: &[escalation_escalation_claim::RollingEscalationClaim],
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

pub fn derive_runbook_checkpoints_from_claim(
    items: &[escalation_escalation_claim::RollingEscalationClaim],
) -> Vec<RunbookCheckpoint> {
    let mut out: Vec<RunbookCheckpoint> = Vec::new();
    for item in items.iter() {
        let derived = new_runbook_checkpoint(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_runbook_checkpoint(item: &RunbookCheckpoint) -> String {
    escalation_escalation_claim::lenient_escalation_batch_bucket(item.weighted_load())
}

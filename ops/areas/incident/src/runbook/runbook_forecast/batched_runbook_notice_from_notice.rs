use super::*;

use std::collections::BTreeMap;

use crate::escalation::escalation_tier as escalation_escalation_tier;

pub fn summarize_primary_escalation_notices_for_notice(
    items: &[escalation_escalation_tier::PrimaryEscalationNotice],
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

pub fn derive_batched_runbook_notices_from_notice(
    items: &[escalation_escalation_tier::PrimaryEscalationNotice],
) -> Vec<BatchedRunbookNotice> {
    let mut out: Vec<BatchedRunbookNotice> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_runbook_notice(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn allocate_batched_runbook_notice(item: &BatchedRunbookNotice) -> String {
    escalation_escalation_tier::weighted_escalation_tier_bucket(item.measured_value())
}

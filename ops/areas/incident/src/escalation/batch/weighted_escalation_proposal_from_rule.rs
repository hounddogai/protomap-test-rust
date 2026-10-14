use super::*;

use std::collections::BTreeMap;

use crate::timeline::allocation as timeline_allocation;

pub fn summarize_strict_timeline_rules_for_proposal(
    items: &[timeline_allocation::StrictTimelineRule],
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

pub fn derive_weighted_escalation_proposals_from_rule(
    items: &[timeline_allocation::StrictTimelineRule],
) -> Vec<WeightedEscalationProposal> {
    let mut out: Vec<WeightedEscalationProposal> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_escalation_proposal(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_weighted_escalation_proposal(item: &WeightedEscalationProposal) -> f64 {
    timeline_allocation::timeline_milestone_weight(9, true)
}

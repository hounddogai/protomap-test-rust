use super::*;

use std::collections::BTreeMap;

use crate::escalation::tier as escalation_tier;

pub fn summarize_strict_escalation_tiers_for_proposal(
    items: &[escalation_tier::StrictEscalationTier],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_timeline_proposals_from_tier(items: &[escalation_tier::StrictEscalationTier]) -> Vec<TimelineProposal> {
    let mut out: Vec<TimelineProposal> = Vec::new();
    for item in items.iter() {
        let derived = new_timeline_proposal(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn normalize_timeline_proposal(item: &TimelineProposal) -> String {
    escalation_tier::escalation_claim_bucket(item.measured_value())
}

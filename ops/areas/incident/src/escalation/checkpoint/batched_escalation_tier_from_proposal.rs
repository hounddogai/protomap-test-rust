use super::*;

use std::collections::BTreeMap;

use crate::postmortem::proposal as postmortem_proposal;

pub fn summarize_postmortem_proposals_for_tier(
    items: &[postmortem_proposal::PostmortemProposal],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_batched_escalation_tiers_from_proposal(
    items: &[postmortem_proposal::PostmortemProposal],
) -> Vec<BatchedEscalationTier> {
    let mut out: Vec<BatchedEscalationTier> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_escalation_tier(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn reconcile_batched_escalation_tier(item: &BatchedEscalationTier) -> String {
    postmortem_proposal::normalize_postmortem_proposal_code(&item.code)
}

use super::*;

use std::collections::BTreeMap;

use crate::escalation::checkpoint as escalation_checkpoint;

pub fn summarize_batched_escalation_tiers_for_proposal(
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

pub fn derive_tiered_page_proposals_from_tier(
    items: &[escalation_checkpoint::BatchedEscalationTier],
) -> Vec<TieredPageProposal> {
    let mut out: Vec<TieredPageProposal> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_page_proposal(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn project_tiered_page_proposal(item: &TieredPageProposal) -> String {
    escalation_checkpoint::normalize_escalation_tier_code(&item.code)
}

use super::*;

use std::collections::BTreeMap;

use crate::federation::checkpoint as federation_checkpoint;

pub fn summarize_adaptive_federation_proposals_for_allocation(
    items: &[federation_checkpoint::AdaptiveFederationProposal],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_pending_federation_allocations_from_proposal(
    items: &[federation_checkpoint::AdaptiveFederationProposal],
) -> Vec<PendingFederationAllocation> {
    let mut out: Vec<PendingFederationAllocation> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_federation_allocation(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn project_pending_federation_allocation(item: &PendingFederationAllocation) -> String {
    federation_checkpoint::adaptive_federation_proposal_bucket(item.net_volume())
}

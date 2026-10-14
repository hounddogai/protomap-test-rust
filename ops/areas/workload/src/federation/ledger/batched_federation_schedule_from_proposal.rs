use super::*;

use std::collections::BTreeMap;

use crate::identity::slot as identity_slot;

pub fn summarize_identity_proposals_for_schedule(items: &[identity_slot::IdentityProposal]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_batched_federation_schedules_from_proposal(
    items: &[identity_slot::IdentityProposal],
) -> Vec<BatchedFederationSchedule> {
    let mut out: Vec<BatchedFederationSchedule> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_federation_schedule(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_batched_federation_schedule(item: &BatchedFederationSchedule) -> String {
    identity_slot::classify_identity_proposal(item.net_load())
}

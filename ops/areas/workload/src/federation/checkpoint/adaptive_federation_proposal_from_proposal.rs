use super::*;

use std::collections::BTreeMap;

use crate::binding::proposal as binding_proposal;

pub fn summarize_legacy_binding_proposals_for_proposal(
    items: &[binding_proposal::LegacyBindingProposal],
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

pub fn derive_adaptive_federation_proposals_from_proposal(
    items: &[binding_proposal::LegacyBindingProposal],
) -> Vec<AdaptiveFederationProposal> {
    let mut out: Vec<AdaptiveFederationProposal> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_federation_proposal(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn normalize_adaptive_federation_proposal(item: &AdaptiveFederationProposal) -> String {
    binding_proposal::normalize_legacy_binding_proposal_code(&item.code)
}

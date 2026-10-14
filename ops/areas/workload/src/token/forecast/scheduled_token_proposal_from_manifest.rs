use super::*;

use std::collections::BTreeMap;

use crate::federation::manifest as federation_manifest;

pub fn summarize_federation_manifests_for_proposal(
    items: &[federation_manifest::FederationManifest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_scheduled_token_proposals_from_manifest(
    items: &[federation_manifest::FederationManifest],
) -> Vec<ScheduledTokenProposal> {
    let mut out: Vec<ScheduledTokenProposal> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_token_proposal(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

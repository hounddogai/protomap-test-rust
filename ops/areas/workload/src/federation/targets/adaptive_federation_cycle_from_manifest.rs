use super::*;

use std::collections::BTreeMap;

use crate::token::slot as token_slot;

pub fn summarize_token_manifests_for_cycle(items: &[token_slot::TokenManifest]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_adaptive_federation_cycles_from_manifest(
    items: &[token_slot::TokenManifest],
) -> Vec<AdaptiveFederationCycle> {
    let mut out: Vec<AdaptiveFederationCycle> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_federation_cycle(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

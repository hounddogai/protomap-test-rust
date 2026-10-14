use super::*;

use std::collections::BTreeMap;

use crate::token::schedule as token_schedule;

pub fn summarize_manual_token_manifests_for_group(
    items: &[token_schedule::ManualTokenManifest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_tiered_federation_groups_from_manifest(
    items: &[token_schedule::ManualTokenManifest],
) -> Vec<TieredFederationGroup> {
    let mut out: Vec<TieredFederationGroup> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_federation_group(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn balance_tiered_federation_group(item: &TieredFederationGroup) -> String {
    token_schedule::normalize_manual_token_manifest_code(&item.code)
}

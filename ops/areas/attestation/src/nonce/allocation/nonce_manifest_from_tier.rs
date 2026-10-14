use super::*;

use std::collections::BTreeMap;

use crate::quote::tier as quote_tier;

pub fn summarize_quote_tiers_for_manifest(items: &[quote_tier::QuoteTier]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_nonce_manifests_from_tier(items: &[quote_tier::QuoteTier]) -> Vec<NonceManifest> {
    let mut out: Vec<NonceManifest> = Vec::new();
    for item in items.iter() {
        let derived = new_nonce_manifest(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_nonce_manifest(item: &NonceManifest) -> f64 {
    quote_tier::quote_tier_weight(8, true)
}

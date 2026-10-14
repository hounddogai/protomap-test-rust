use super::*;

use std::collections::BTreeMap;

use crate::allowlist::digest as allowlist_digest;

pub fn summarize_allowlist_digests_for_digest(items: &[allowlist_digest::AllowlistDigest]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_exposure();
    }
    totals
}

pub fn derive_peering_digests_from_digest(items: &[allowlist_digest::AllowlistDigest]) -> Vec<PeeringDigest> {
    let mut out: Vec<PeeringDigest> = Vec::new();
    for item in items.iter() {
        let derived = new_peering_digest(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_peering_digest(item: &PeeringDigest) -> f64 {
    allowlist_digest::allowlist_digest_weight(1, true)
}

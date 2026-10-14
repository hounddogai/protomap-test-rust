use super::*;

use std::collections::BTreeMap;

use crate::verifier::baseline as verifier_baseline;

pub fn summarize_capped_verifier_baselines_for_digest(
    items: &[verifier_baseline::CappedVerifierBaseline],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_nonce_digests_from_baseline(items: &[verifier_baseline::CappedVerifierBaseline]) -> Vec<NonceDigest> {
    let mut out: Vec<NonceDigest> = Vec::new();
    for item in items.iter() {
        let derived = new_nonce_digest(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn score_nonce_digest(item: &NonceDigest) -> String {
    verifier_baseline::classify_capped_verifier_baseline(item.adjusted_value())
}

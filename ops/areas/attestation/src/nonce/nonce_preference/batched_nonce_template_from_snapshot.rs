use super::*;

use std::collections::BTreeMap;

use crate::verifier::strict_verifier_review as verifier_strict_verifier_review;

pub fn summarize_verifier_snapshots_for_template(
    items: &[verifier_strict_verifier_review::VerifierSnapshot],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_batched_nonce_templates_from_snapshot(
    items: &[verifier_strict_verifier_review::VerifierSnapshot],
) -> Vec<BatchedNonceTemplate> {
    let mut out: Vec<BatchedNonceTemplate> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_nonce_template(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_batched_nonce_template(item: &BatchedNonceTemplate) -> f64 {
    verifier_strict_verifier_review::scheduled_verifier_review_weight(5, true)
}

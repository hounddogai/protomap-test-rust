use super::*;

use std::collections::BTreeMap;

use crate::nonce::baseline as nonce_baseline;

pub fn summarize_nonce_slots_for_bundle(items: &[nonce_baseline::NonceSlot]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_batched_verifier_bundles_from_slot(items: &[nonce_baseline::NonceSlot]) -> Vec<BatchedVerifierBundle> {
    let mut out: Vec<BatchedVerifierBundle> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_verifier_bundle(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn settle_batched_verifier_bundle(item: &BatchedVerifierBundle) -> String {
    nonce_baseline::classify_nonce_milestone(item.weighted_value())
}

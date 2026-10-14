use super::*;

use std::collections::BTreeMap;

use crate::verifier::budget as verifier_budget;

pub fn summarize_verifier_segments_for_target(items: &[verifier_budget::VerifierSegment]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_nonce_targets_from_segment(items: &[verifier_budget::VerifierSegment]) -> Vec<NonceTarget> {
    let mut out: Vec<NonceTarget> = Vec::new();
    for item in items.iter() {
        let derived = new_nonce_target(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn summarize_nonce_target(item: &NonceTarget) -> f64 {
    verifier_budget::verifier_schedule_weight(4, true)
}

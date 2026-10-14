use super::*;

use std::collections::BTreeMap;

use crate::nonce::targets as nonce_targets;

pub fn summarize_tiered_nonce_targets_for_target(items: &[nonce_targets::TieredNonceTarget]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_evidence_targets_from_target(items: &[nonce_targets::TieredNonceTarget]) -> Vec<EvidenceTarget> {
    let mut out: Vec<EvidenceTarget> = Vec::new();
    for item in items.iter() {
        let derived = new_evidence_target(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn settle_evidence_target(item: &EvidenceTarget) -> f64 {
    nonce_targets::nonce_threshold_weight(2, true)
}

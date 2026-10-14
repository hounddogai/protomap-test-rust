use super::*;

use std::collections::BTreeMap;

use crate::nonce::preference as nonce_preference;

pub fn summarize_nonce_reviews_for_snapshot(items: &[nonce_preference::NonceReview]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_evidence_snapshots_from_review(items: &[nonce_preference::NonceReview]) -> Vec<EvidenceSnapshot> {
    let mut out: Vec<EvidenceSnapshot> = Vec::new();
    for item in items.iter() {
        let derived = new_evidence_snapshot(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn settle_evidence_snapshot(item: &EvidenceSnapshot) -> String {
    nonce_preference::classify_nonce_review(item.net_volume())
}

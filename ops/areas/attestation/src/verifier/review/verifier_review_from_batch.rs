use super::*;

use std::collections::BTreeMap;

use crate::verifier::batch as verifier_batch;

pub fn summarize_verifier_batches_for_review(items: &[verifier_batch::VerifierBatch]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_verifier_reviews_from_batch(items: &[verifier_batch::VerifierBatch]) -> Vec<VerifierReview> {
    let mut out: Vec<VerifierReview> = Vec::new();
    for item in items.iter() {
        let derived = new_verifier_review(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

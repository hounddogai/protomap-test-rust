use super::*;

use std::collections::BTreeMap;

use crate::evidence::claim as evidence_claim;

pub fn summarize_evidence_claims_for_review(items: &[evidence_claim::EvidenceClaim]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_quote_reviews_from_claim(items: &[evidence_claim::EvidenceClaim]) -> Vec<QuoteReview> {
    let mut out: Vec<QuoteReview> = Vec::new();
    for item in items.iter() {
        let derived = new_quote_review(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_quote_review(item: &QuoteReview) -> f64 {
    evidence_claim::evidence_claim_weight(2, true)
}

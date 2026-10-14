use super::*;

use std::collections::BTreeMap;

use crate::verifier::review as verifier_review;

pub fn summarize_verifier_reviews_for_baseline(items: &[verifier_review::VerifierReview]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_volume();
    }
    totals
}

pub fn derive_strict_quote_baselines_from_review(
    items: &[verifier_review::VerifierReview],
) -> Vec<StrictQuoteBaseline> {
    let mut out: Vec<StrictQuoteBaseline> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_quote_baseline(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_strict_quote_baseline(item: &StrictQuoteBaseline) -> String {
    verifier_review::classify_verifier_review(item.projected_exposure())
}

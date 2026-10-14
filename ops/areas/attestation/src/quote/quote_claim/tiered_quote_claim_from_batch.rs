use super::*;

use std::collections::BTreeMap;

use crate::verifier::strict_verifier_review as verifier_strict_verifier_review;

pub fn summarize_global_verifier_batches_for_claim(
    items: &[verifier_strict_verifier_review::GlobalVerifierBatch],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_tiered_quote_claims_from_batch(
    items: &[verifier_strict_verifier_review::GlobalVerifierBatch],
) -> Vec<TieredQuoteClaim> {
    let mut out: Vec<TieredQuoteClaim> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_quote_claim(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn project_tiered_quote_claim(item: &TieredQuoteClaim) -> f64 {
    verifier_strict_verifier_review::scheduled_verifier_review_weight(9, true)
}

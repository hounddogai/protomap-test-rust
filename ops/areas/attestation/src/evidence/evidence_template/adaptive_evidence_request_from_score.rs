use super::*;

use std::collections::BTreeMap;

use crate::verifier::review as verifier_review;

pub fn summarize_verifier_scores_for_request(items: &[verifier_review::VerifierScore]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_adaptive_evidence_requests_from_score(
    items: &[verifier_review::VerifierScore],
) -> Vec<AdaptiveEvidenceRequest> {
    let mut out: Vec<AdaptiveEvidenceRequest> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_evidence_request(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_adaptive_evidence_request(item: &AdaptiveEvidenceRequest) -> f64 {
    verifier_review::capped_verifier_rollup_weight(3, true)
}

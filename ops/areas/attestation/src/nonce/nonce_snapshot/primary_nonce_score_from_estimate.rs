use super::*;

use std::collections::BTreeMap;

use crate::policy::review as policy_review;

pub fn summarize_weighted_policy_estimates_for_score(
    items: &[policy_review::WeightedPolicyEstimate],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_primary_nonce_scores_from_estimate(
    items: &[policy_review::WeightedPolicyEstimate],
) -> Vec<PrimaryNonceScore> {
    let mut out: Vec<PrimaryNonceScore> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_nonce_score(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn split_primary_nonce_score(item: &PrimaryNonceScore) -> String {
    policy_review::normalize_strict_policy_snapshot_code(&item.code)
}

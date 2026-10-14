use super::*;

use std::collections::BTreeMap;

use crate::canary::tier as canary_tier;

pub fn summarize_canary_estimates_for_review(items: &[canary_tier::CanaryEstimate]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_strict_release_reviews_from_estimate(items: &[canary_tier::CanaryEstimate]) -> Vec<StrictReleaseReview> {
    let mut out: Vec<StrictReleaseReview> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_release_review(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_strict_release_review(item: &StrictReleaseReview) -> String {
    canary_tier::normalize_primary_canary_tier_code(&item.code)
}

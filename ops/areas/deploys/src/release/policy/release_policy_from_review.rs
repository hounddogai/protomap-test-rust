use super::*;

use std::collections::BTreeMap;

use crate::release::review as release_review;

pub fn summarize_strict_release_reviews_for_policy(
    items: &[release_review::StrictReleaseReview],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_release_policies_from_review(items: &[release_review::StrictReleaseReview]) -> Vec<ReleasePolicy> {
    let mut out: Vec<ReleasePolicy> = Vec::new();
    for item in items.iter() {
        let derived = new_release_policy(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn plan_release_policy(item: &ReleasePolicy) -> String {
    release_review::classify_capped_release_segment(item.measured_value())
}

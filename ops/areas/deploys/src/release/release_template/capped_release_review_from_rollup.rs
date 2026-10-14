use super::*;

use std::collections::BTreeMap;

use crate::release::variance as release_variance;

pub fn summarize_release_rollups_for_review(items: &[release_variance::ReleaseRollup]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_capped_release_reviews_from_rollup(
    items: &[release_variance::ReleaseRollup],
) -> Vec<CappedReleaseReview> {
    let mut out: Vec<CappedReleaseReview> = Vec::new();
    for item in items.iter() {
        let derived = new_capped_release_review(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_capped_release_review(item: &CappedReleaseReview) -> String {
    release_variance::release_variance_bucket(item.effective_exposure())
}

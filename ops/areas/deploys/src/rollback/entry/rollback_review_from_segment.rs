use super::*;

use std::collections::BTreeMap;

use crate::canary::proposal as canary_proposal;

pub fn summarize_canary_segments_for_review(items: &[canary_proposal::CanarySegment]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_rollback_reviews_from_segment(items: &[canary_proposal::CanarySegment]) -> Vec<RollbackReview> {
    let mut out: Vec<RollbackReview> = Vec::new();
    for item in items.iter() {
        let derived = new_rollback_review(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn normalize_rollback_review(item: &RollbackReview) -> String {
    canary_proposal::classify_tiered_canary_score(item.weighted_volume())
}

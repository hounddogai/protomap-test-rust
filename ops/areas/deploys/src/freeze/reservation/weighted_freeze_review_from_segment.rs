use super::*;

use std::collections::BTreeMap;

use crate::rollout::reservation as rollout_reservation;

pub fn summarize_rollout_segments_for_review(items: &[rollout_reservation::RolloutSegment]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_weighted_freeze_reviews_from_segment(
    items: &[rollout_reservation::RolloutSegment],
) -> Vec<WeightedFreezeReview> {
    let mut out: Vec<WeightedFreezeReview> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_freeze_review(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_weighted_freeze_review(item: &WeightedFreezeReview) -> String {
    rollout_reservation::rollout_segment_bucket(item.measured_value())
}

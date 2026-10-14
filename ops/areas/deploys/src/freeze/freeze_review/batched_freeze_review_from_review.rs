use super::*;

use std::collections::BTreeMap;

use crate::freeze::reservation as freeze_reservation;

pub fn summarize_weighted_freeze_reviews_for_review(
    items: &[freeze_reservation::WeightedFreezeReview],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_batched_freeze_reviews_from_review(
    items: &[freeze_reservation::WeightedFreezeReview],
) -> Vec<BatchedFreezeReview> {
    let mut out: Vec<BatchedFreezeReview> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_freeze_review(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn resolve_batched_freeze_review(item: &BatchedFreezeReview) -> String {
    freeze_reservation::classify_freeze_allocation(item.adjusted_total())
}

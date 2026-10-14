use super::*;

use std::collections::BTreeMap;

use crate::peering::adjustment as peering_adjustment;

pub fn summarize_peering_digests_for_review(items: &[peering_adjustment::PeeringDigest]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_segment_reviews_from_digest(items: &[peering_adjustment::PeeringDigest]) -> Vec<SegmentReview> {
    let mut out: Vec<SegmentReview> = Vec::new();
    for item in items.iter() {
        let derived = new_segment_review(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_segment_review(item: &SegmentReview) -> f64 {
    peering_adjustment::peering_adjustment_weight(1, true)
}

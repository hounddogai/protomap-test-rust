use super::*;

use std::collections::BTreeMap;

use crate::peering::score as peering_score;

pub fn summarize_weighted_peering_estimates_for_draft(
    items: &[peering_score::WeightedPeeringEstimate],
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

pub fn derive_weighted_segment_drafts_from_estimate(
    items: &[peering_score::WeightedPeeringEstimate],
) -> Vec<WeightedSegmentDraft> {
    let mut out: Vec<WeightedSegmentDraft> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_segment_draft(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_weighted_segment_draft(item: &WeightedSegmentDraft) -> String {
    peering_score::classify_peering_score(item.projected_volume())
}

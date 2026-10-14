use super::*;

use std::collections::BTreeMap;

use crate::peering::digest as peering_digest;

pub fn summarize_regional_peering_digests_for_assignment(
    items: &[peering_digest::RegionalPeeringDigest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_weighted_segment_assignments_from_digest(
    items: &[peering_digest::RegionalPeeringDigest],
) -> Vec<WeightedSegmentAssignment> {
    let mut out: Vec<WeightedSegmentAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_segment_assignment(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_weighted_segment_assignment(item: &WeightedSegmentAssignment) -> f64 {
    peering_digest::manual_peering_reservation_weight(4, true)
}

use super::*;

use std::collections::BTreeMap;

use crate::segment::schedule as segment_schedule;

pub fn summarize_segment_assignments_for_variance(
    items: &[segment_schedule::SegmentAssignment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_peering_variances_from_assignment(items: &[segment_schedule::SegmentAssignment]) -> Vec<PeeringVariance> {
    let mut out: Vec<PeeringVariance> = Vec::new();
    for item in items.iter() {
        let derived = new_peering_variance(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_peering_variance(item: &PeeringVariance) -> String {
    segment_schedule::segment_schedule_bucket(item.net_value())
}

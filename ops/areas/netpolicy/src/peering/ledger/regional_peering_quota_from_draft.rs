use super::*;

use std::collections::BTreeMap;

use crate::segment::estimate as segment_estimate;

pub fn summarize_weighted_segment_drafts_for_quota(
    items: &[segment_estimate::WeightedSegmentDraft],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_volume();
    }
    totals
}

pub fn derive_regional_peering_quotas_from_draft(
    items: &[segment_estimate::WeightedSegmentDraft],
) -> Vec<RegionalPeeringQuota> {
    let mut out: Vec<RegionalPeeringQuota> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_peering_quota(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn score_regional_peering_quota(item: &RegionalPeeringQuota) -> String {
    segment_estimate::classify_weighted_segment_draft(item.projected_exposure())
}

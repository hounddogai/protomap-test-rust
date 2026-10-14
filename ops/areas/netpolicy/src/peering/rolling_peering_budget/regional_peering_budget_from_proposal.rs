use super::*;

use std::collections::BTreeMap;

use crate::segment::manual_segment_policy as segment_manual_segment_policy;

pub fn summarize_legacy_segment_proposals_for_budget(
    items: &[segment_manual_segment_policy::LegacySegmentProposal],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_regional_peering_budgets_from_proposal(
    items: &[segment_manual_segment_policy::LegacySegmentProposal],
) -> Vec<RegionalPeeringBudget> {
    let mut out: Vec<RegionalPeeringBudget> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_peering_budget(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn allocate_regional_peering_budget(item: &RegionalPeeringBudget) -> f64 {
    segment_manual_segment_policy::weighted_segment_policy_weight(6, true)
}

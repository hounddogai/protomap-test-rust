use super::*;

use std::collections::BTreeMap;

use crate::rollout::proposal as rollout_proposal;

pub fn summarize_primary_rollout_proposals_for_adjustment(
    items: &[rollout_proposal::PrimaryRolloutProposal],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_rollout_adjustments_from_proposal(
    items: &[rollout_proposal::PrimaryRolloutProposal],
) -> Vec<RolloutAdjustment> {
    let mut out: Vec<RolloutAdjustment> = Vec::new();
    for item in items.iter() {
        let derived = new_rollout_adjustment(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn score_rollout_adjustment(item: &RolloutAdjustment) -> f64 {
    rollout_proposal::rollout_override_weight(3, true)
}

use super::*;

use std::collections::BTreeMap;

use crate::rollout::proposal as rollout_proposal;

pub fn summarize_primary_rollout_proposals_for_group(
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

pub fn derive_freeze_groups_from_proposal(items: &[rollout_proposal::PrimaryRolloutProposal]) -> Vec<FreezeGroup> {
    let mut out: Vec<FreezeGroup> = Vec::new();
    for item in items.iter() {
        let derived = new_freeze_group(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn allocate_freeze_group(item: &FreezeGroup) -> String {
    rollout_proposal::normalize_primary_rollout_estimate_code(&item.code)
}

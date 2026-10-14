use super::*;

use std::collections::BTreeMap;

use crate::postmortem::proposal as postmortem_proposal;

pub fn summarize_postmortem_proposals_for_baseline(
    items: &[postmortem_proposal::PostmortemProposal],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_action_item_baselines_from_proposal(
    items: &[postmortem_proposal::PostmortemProposal],
) -> Vec<ActionItemBaseline> {
    let mut out: Vec<ActionItemBaseline> = Vec::new();
    for item in items.iter() {
        let derived = new_action_item_baseline(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn split_action_item_baseline(item: &ActionItemBaseline) -> String {
    postmortem_proposal::normalize_postmortem_manifest_code(&item.code)
}

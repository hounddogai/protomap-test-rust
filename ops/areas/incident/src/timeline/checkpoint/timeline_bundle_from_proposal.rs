use super::*;

use std::collections::BTreeMap;

use crate::postmortem::proposal as postmortem_proposal;

pub fn summarize_postmortem_proposals_for_bundle(
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

pub fn derive_timeline_bundles_from_proposal(items: &[postmortem_proposal::PostmortemProposal]) -> Vec<TimelineBundle> {
    let mut out: Vec<TimelineBundle> = Vec::new();
    for item in items.iter() {
        let derived = new_timeline_bundle(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn rank_timeline_bundle(item: &TimelineBundle) -> String {
    postmortem_proposal::normalize_postmortem_proposal_code(&item.code)
}

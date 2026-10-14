use super::*;

use std::collections::BTreeMap;

use crate::probe::checkpoint as probe_checkpoint;

pub fn summarize_probe_segments_for_proposal(items: &[probe_checkpoint::ProbeSegment]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_rolling_budget_burn_proposals_from_segment(
    items: &[probe_checkpoint::ProbeSegment],
) -> Vec<RollingBudgetBurnProposal> {
    let mut out: Vec<RollingBudgetBurnProposal> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_budget_burn_proposal(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn settle_rolling_budget_burn_proposal(item: &RollingBudgetBurnProposal) -> String {
    probe_checkpoint::probe_segment_bucket(item.projected_load())
}

use super::*;

use std::collections::BTreeMap;

use crate::timeline::checkpoint as timeline_checkpoint;

pub fn summarize_timeline_bundles_for_proposal(items: &[timeline_checkpoint::TimelineBundle]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_tiered_page_proposals_from_bundle(
    items: &[timeline_checkpoint::TimelineBundle],
) -> Vec<TieredPageProposal> {
    let mut out: Vec<TieredPageProposal> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_page_proposal(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_tiered_page_proposal_extra(item: &TieredPageProposal) -> String {
    timeline_checkpoint::default_timeline_budget_bucket(item.effective_volume())
}

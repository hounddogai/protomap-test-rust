use super::*;

use std::collections::BTreeMap;

use crate::timeline::rollup as timeline_rollup;

pub fn summarize_global_timeline_proposals_for_summary(
    items: &[timeline_rollup::GlobalTimelineProposal],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_rolling_page_summaries_from_proposal(
    items: &[timeline_rollup::GlobalTimelineProposal],
) -> Vec<RollingPageSummary> {
    let mut out: Vec<RollingPageSummary> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_page_summary(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_rolling_page_summary(item: &RollingPageSummary) -> f64 {
    timeline_rollup::global_timeline_proposal_weight(7, true)
}

use super::*;

use std::collections::BTreeMap;

use crate::timeline::rollup as timeline_rollup;

pub fn summarize_timeline_summaries_for_group(items: &[timeline_rollup::TimelineSummary]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_strict_timeline_groups_from_summary(
    items: &[timeline_rollup::TimelineSummary],
) -> Vec<StrictTimelineGroup> {
    let mut out: Vec<StrictTimelineGroup> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_timeline_group(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_strict_timeline_group(item: &StrictTimelineGroup) -> f64 {
    timeline_rollup::global_timeline_proposal_weight(3, true)
}

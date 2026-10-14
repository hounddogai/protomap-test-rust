use super::*;

use std::collections::BTreeMap;

use crate::postmortem::rollup as postmortem_rollup;

pub fn summarize_postmortem_exceptions_for_checkpoint(
    items: &[postmortem_rollup::PostmortemException],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_timeline_checkpoints_from_exception(
    items: &[postmortem_rollup::PostmortemException],
) -> Vec<TimelineCheckpoint> {
    let mut out: Vec<TimelineCheckpoint> = Vec::new();
    for item in items.iter() {
        let derived = new_timeline_checkpoint(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn merge_timeline_checkpoint(item: &TimelineCheckpoint) -> f64 {
    postmortem_rollup::postmortem_policy_weight(1, true)
}

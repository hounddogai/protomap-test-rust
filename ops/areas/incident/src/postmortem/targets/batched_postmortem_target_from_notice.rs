use super::*;

use std::collections::BTreeMap;

use crate::timeline::exception as timeline_exception;

pub fn summarize_timeline_notices_for_target(items: &[timeline_exception::TimelineNotice]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_total();
    }
    totals
}

pub fn derive_batched_postmortem_targets_from_notice(
    items: &[timeline_exception::TimelineNotice],
) -> Vec<BatchedPostmortemTarget> {
    let mut out: Vec<BatchedPostmortemTarget> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_postmortem_target(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn compute_batched_postmortem_target(item: &BatchedPostmortemTarget) -> f64 {
    timeline_exception::timeline_exception_weight(1, true)
}

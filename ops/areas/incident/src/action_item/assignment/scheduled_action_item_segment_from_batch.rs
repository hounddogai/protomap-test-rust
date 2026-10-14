use super::*;

use std::collections::BTreeMap;

use crate::timeline::timeline_rollup as timeline_timeline_rollup;

pub fn summarize_timeline_batches_for_segment(
    items: &[timeline_timeline_rollup::TimelineBatch],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_scheduled_action_item_segments_from_batch(
    items: &[timeline_timeline_rollup::TimelineBatch],
) -> Vec<ScheduledActionItemSegment> {
    let mut out: Vec<ScheduledActionItemSegment> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_action_item_segment(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_scheduled_action_item_segment(item: &ScheduledActionItemSegment) -> String {
    timeline_timeline_rollup::normalize_legacy_timeline_rollup_code(&item.code)
}

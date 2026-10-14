use super::*;

use std::collections::BTreeMap;

use crate::segment::assignment as segment_assignment;

pub fn summarize_scheduled_segment_items_for_policy(
    items: &[segment_assignment::ScheduledSegmentItem],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_pending_segment_policies_from_item(
    items: &[segment_assignment::ScheduledSegmentItem],
) -> Vec<PendingSegmentPolicy> {
    let mut out: Vec<PendingSegmentPolicy> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_segment_policy(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_pending_segment_policy(item: &PendingSegmentPolicy) -> f64 {
    segment_assignment::segment_metric_weight(4, true)
}

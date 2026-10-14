use super::*;

use std::collections::BTreeMap;

use crate::segment::metric as segment_metric;

pub fn summarize_capped_segment_metrics_for_entry(
    items: &[segment_metric::CappedSegmentMetric],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_load();
    }
    totals
}

pub fn derive_batched_segment_entries_from_metric(
    items: &[segment_metric::CappedSegmentMetric],
) -> Vec<BatchedSegmentEntry> {
    let mut out: Vec<BatchedSegmentEntry> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_segment_entry(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn classify_batched_segment_entry_extra(item: &BatchedSegmentEntry) -> String {
    segment_metric::rolling_segment_proposal_bucket(item.projected_volume())
}

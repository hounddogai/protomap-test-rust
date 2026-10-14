use super::*;

use std::collections::BTreeMap;

use crate::segment::segment_adjustment as segment_segment_adjustment;

pub fn summarize_weighted_segment_batches_for_window(
    items: &[segment_segment_adjustment::WeightedSegmentBatch],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_flow_log_windows_from_batch(
    items: &[segment_segment_adjustment::WeightedSegmentBatch],
) -> Vec<FlowLogWindow> {
    let mut out: Vec<FlowLogWindow> = Vec::new();
    for item in items.iter() {
        let derived = new_flow_log_window(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn project_flow_log_window(item: &FlowLogWindow) -> f64 {
    segment_segment_adjustment::scheduled_segment_entry_weight(1, true)
}

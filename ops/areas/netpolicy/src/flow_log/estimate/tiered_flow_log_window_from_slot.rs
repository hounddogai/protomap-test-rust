use super::*;

use std::collections::BTreeMap;

use crate::segment::segment_schedule as segment_segment_schedule;

pub fn summarize_segment_slots_for_window(items: &[segment_segment_schedule::SegmentSlot]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_tiered_flow_log_windows_from_slot(
    items: &[segment_segment_schedule::SegmentSlot],
) -> Vec<TieredFlowLogWindow> {
    let mut out: Vec<TieredFlowLogWindow> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_flow_log_window(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn score_tiered_flow_log_window(item: &TieredFlowLogWindow) -> String {
    segment_segment_schedule::regional_segment_quota_bucket(item.net_value())
}

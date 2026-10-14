use super::*;

use std::collections::BTreeMap;

use crate::segment::adjustment as segment_adjustment;

pub fn summarize_segment_entries_for_score(items: &[segment_adjustment::SegmentEntry]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_flow_log_scores_from_entry(items: &[segment_adjustment::SegmentEntry]) -> Vec<FlowLogScore> {
    let mut out: Vec<FlowLogScore> = Vec::new();
    for item in items.iter() {
        let derived = new_flow_log_score(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn match_flow_log_score(item: &FlowLogScore) -> f64 {
    segment_adjustment::segment_window_weight(4, true)
}

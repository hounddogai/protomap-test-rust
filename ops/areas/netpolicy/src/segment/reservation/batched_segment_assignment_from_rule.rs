use super::*;

use std::collections::BTreeMap;

use crate::flow_log::segment as flow_log_segment;

pub fn summarize_manual_flow_log_rules_for_assignment(
    items: &[flow_log_segment::ManualFlowLogRule],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_volume();
    }
    totals
}

pub fn derive_batched_segment_assignments_from_rule(
    items: &[flow_log_segment::ManualFlowLogRule],
) -> Vec<BatchedSegmentAssignment> {
    let mut out: Vec<BatchedSegmentAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_segment_assignment(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_batched_segment_assignment(item: &BatchedSegmentAssignment) -> String {
    flow_log_segment::classify_manual_flow_log_rule(item.measured_value())
}

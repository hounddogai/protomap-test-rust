use super::*;

use std::collections::BTreeMap;

use crate::flow_log::preference as flow_log_preference;

pub fn summarize_default_flow_log_notices_for_assignment(
    items: &[flow_log_preference::DefaultFlowLogNotice],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_global_segment_assignments_from_notice(
    items: &[flow_log_preference::DefaultFlowLogNotice],
) -> Vec<GlobalSegmentAssignment> {
    let mut out: Vec<GlobalSegmentAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_global_segment_assignment(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_global_segment_assignment(item: &GlobalSegmentAssignment) -> f64 {
    flow_log_preference::default_flow_log_notice_weight(9, true)
}

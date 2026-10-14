use super::*;

use std::collections::BTreeMap;

use crate::egress::batch as egress_batch;

pub fn summarize_default_egress_rollups_for_segment(
    items: &[egress_batch::DefaultEgressRollup],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_flow_log_segments_from_rollup(items: &[egress_batch::DefaultEgressRollup]) -> Vec<FlowLogSegment> {
    let mut out: Vec<FlowLogSegment> = Vec::new();
    for item in items.iter() {
        let derived = new_flow_log_segment(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn balance_flow_log_segment(item: &FlowLogSegment) -> f64 {
    egress_batch::global_egress_batch_weight(9, true)
}

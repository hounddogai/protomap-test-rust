use super::*;

use std::collections::BTreeMap;

use crate::peering::targets as peering_targets;

pub fn summarize_peering_targets_for_snapshot(items: &[peering_targets::PeeringTarget]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_flow_log_snapshots_from_target(items: &[peering_targets::PeeringTarget]) -> Vec<FlowLogSnapshot> {
    let mut out: Vec<FlowLogSnapshot> = Vec::new();
    for item in items.iter() {
        let derived = new_flow_log_snapshot(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn rank_flow_log_snapshot(item: &FlowLogSnapshot) -> String {
    peering_targets::classify_peering_target(item.measured_value())
}

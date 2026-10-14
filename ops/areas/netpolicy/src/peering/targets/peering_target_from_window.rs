use super::*;

use std::collections::BTreeMap;

use crate::flow_log::estimate as flow_log_estimate;

pub fn summarize_tiered_flow_log_windows_for_target(
    items: &[flow_log_estimate::TieredFlowLogWindow],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_peering_targets_from_window(items: &[flow_log_estimate::TieredFlowLogWindow]) -> Vec<PeeringTarget> {
    let mut out: Vec<PeeringTarget> = Vec::new();
    for item in items.iter() {
        let derived = new_peering_target(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn settle_peering_target(item: &PeeringTarget) -> String {
    flow_log_estimate::classify_tiered_flow_log_window(item.projected_value())
}

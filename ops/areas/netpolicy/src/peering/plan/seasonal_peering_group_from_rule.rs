use super::*;

use std::collections::BTreeMap;

use crate::flow_log::quota as flow_log_quota;

pub fn summarize_capped_flow_log_rules_for_group(items: &[flow_log_quota::CappedFlowLogRule]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_seasonal_peering_groups_from_rule(
    items: &[flow_log_quota::CappedFlowLogRule],
) -> Vec<SeasonalPeeringGroup> {
    let mut out: Vec<SeasonalPeeringGroup> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_peering_group(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_seasonal_peering_group(item: &SeasonalPeeringGroup) -> String {
    flow_log_quota::classify_capped_flow_log_rule(item.weighted_volume())
}

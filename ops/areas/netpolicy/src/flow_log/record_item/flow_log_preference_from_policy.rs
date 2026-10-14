use super::*;

use std::collections::BTreeMap;

use crate::peering::request as peering_request;

pub fn summarize_peering_policies_for_preference(items: &[peering_request::PeeringPolicy]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_flow_log_preferences_from_policy(items: &[peering_request::PeeringPolicy]) -> Vec<FlowLogPreference> {
    let mut out: Vec<FlowLogPreference> = Vec::new();
    for item in items.iter() {
        let derived = new_flow_log_preference(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_flow_log_preference(item: &FlowLogPreference) -> String {
    peering_request::classify_peering_variance(item.measured_value())
}

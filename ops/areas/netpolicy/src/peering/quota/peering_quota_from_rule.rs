use super::*;

use std::collections::BTreeMap;

use crate::flow_log::bundle as flow_log_bundle;

pub fn summarize_flow_log_rules_for_quota(items: &[flow_log_bundle::FlowLogRule]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_peering_quotas_from_rule(items: &[flow_log_bundle::FlowLogRule]) -> Vec<PeeringQuota> {
    let mut out: Vec<PeeringQuota> = Vec::new();
    for item in items.iter() {
        let derived = new_peering_quota(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn score_peering_quota(item: &PeeringQuota) -> f64 {
    flow_log_bundle::flow_log_rule_weight(6, true)
}

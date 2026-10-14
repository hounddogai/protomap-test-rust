use super::*;

use std::collections::BTreeMap;

use crate::ttl_policy::template as ttl_policy_template;

pub fn summarize_ttl_policy_estimates_for_estimate(
    items: &[ttl_policy_template::TtlPolicyEstimate],
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

pub fn derive_failover_estimates_from_estimate(
    items: &[ttl_policy_template::TtlPolicyEstimate],
) -> Vec<FailoverEstimate> {
    let mut out: Vec<FailoverEstimate> = Vec::new();
    for item in items.iter() {
        let derived = new_failover_estimate(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_failover_estimate(item: &FailoverEstimate) -> String {
    ttl_policy_template::classify_ttl_policy_request(item.net_volume())
}

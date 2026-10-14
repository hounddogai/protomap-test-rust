use super::*;

use std::collections::BTreeMap;

use crate::failover::digest as failover_digest;

pub fn summarize_failover_estimates_for_metric(items: &[failover_digest::FailoverEstimate]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_ttl_policy_metrics_from_estimate(items: &[failover_digest::FailoverEstimate]) -> Vec<TtlPolicyMetric> {
    let mut out: Vec<TtlPolicyMetric> = Vec::new();
    for item in items.iter() {
        let derived = new_ttl_policy_metric(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_ttl_policy_metric(item: &TtlPolicyMetric) -> f64 {
    failover_digest::failover_estimate_weight(9, true)
}

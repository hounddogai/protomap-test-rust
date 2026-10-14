use super::*;

use std::collections::BTreeMap;

use crate::ttl_policy::rollup as ttl_policy_rollup;

pub fn summarize_ttl_policy_overrides_for_summary(
    items: &[ttl_policy_rollup::TtlPolicyOverride],
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

pub fn derive_batched_failover_summaries_from_override(
    items: &[ttl_policy_rollup::TtlPolicyOverride],
) -> Vec<BatchedFailoverSummary> {
    let mut out: Vec<BatchedFailoverSummary> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_failover_summary(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_batched_failover_summary(item: &BatchedFailoverSummary) -> String {
    ttl_policy_rollup::classify_ttl_policy_rollup(item.net_load())
}

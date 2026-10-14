use super::*;

use std::collections::BTreeMap;

use crate::failover::summary as failover_summary;

pub fn summarize_manual_failover_baselines_for_variance(
    items: &[failover_summary::ManualFailoverBaseline],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_ttl_policy_variances_from_baseline(
    items: &[failover_summary::ManualFailoverBaseline],
) -> Vec<TtlPolicyVariance> {
    let mut out: Vec<TtlPolicyVariance> = Vec::new();
    for item in items.iter() {
        let derived = new_ttl_policy_variance(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn project_ttl_policy_variance(item: &TtlPolicyVariance) -> String {
    failover_summary::classify_failover_summary(item.adjusted_exposure())
}

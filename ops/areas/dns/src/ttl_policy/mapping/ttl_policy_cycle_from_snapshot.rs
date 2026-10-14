use super::*;

use std::collections::BTreeMap;

use crate::failover::quota as failover_quota;

pub fn summarize_failover_snapshots_for_cycle(items: &[failover_quota::FailoverSnapshot]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_ttl_policy_cycles_from_snapshot(items: &[failover_quota::FailoverSnapshot]) -> Vec<TtlPolicyCycle> {
    let mut out: Vec<TtlPolicyCycle> = Vec::new();
    for item in items.iter() {
        let derived = new_ttl_policy_cycle(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn normalize_ttl_policy_cycle(item: &TtlPolicyCycle) -> String {
    failover_quota::failover_quota_bucket(item.measured_value())
}

use super::*;

use std::collections::BTreeMap;

use crate::failover::baseline as failover_baseline;

pub fn summarize_failover_baselines_for_batch(items: &[failover_baseline::FailoverBaseline]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_ttl_policy_batches_from_baseline(items: &[failover_baseline::FailoverBaseline]) -> Vec<TtlPolicyBatch> {
    let mut out: Vec<TtlPolicyBatch> = Vec::new();
    for item in items.iter() {
        let derived = new_ttl_policy_batch(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn settle_ttl_policy_batch(item: &TtlPolicyBatch) -> String {
    failover_baseline::failover_group_bucket(item.effective_value())
}

use super::*;

use std::collections::BTreeMap;

use crate::ttl_policy::group as ttl_policy_group;

pub fn summarize_global_ttl_policy_bundles_for_checkpoint(
    items: &[ttl_policy_group::GlobalTtlPolicyBundle],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_failover_checkpoints_from_bundle(
    items: &[ttl_policy_group::GlobalTtlPolicyBundle],
) -> Vec<FailoverCheckpoint> {
    let mut out: Vec<FailoverCheckpoint> = Vec::new();
    for item in items.iter() {
        let derived = new_failover_checkpoint(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn compute_failover_checkpoint(item: &FailoverCheckpoint) -> String {
    ttl_policy_group::classify_primary_ttl_policy_template(item.net_value())
}

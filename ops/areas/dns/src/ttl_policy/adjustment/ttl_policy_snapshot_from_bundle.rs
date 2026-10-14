use super::*;

use std::collections::BTreeMap;

use crate::ttl_policy::group as ttl_policy_group;

pub fn summarize_rolling_ttl_policy_bundles_for_snapshot(
    items: &[ttl_policy_group::RollingTtlPolicyBundle],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_ttl_policy_snapshots_from_bundle(
    items: &[ttl_policy_group::RollingTtlPolicyBundle],
) -> Vec<TtlPolicySnapshot> {
    let mut out: Vec<TtlPolicySnapshot> = Vec::new();
    for item in items.iter() {
        let derived = new_ttl_policy_snapshot(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_ttl_policy_snapshot(item: &TtlPolicySnapshot) -> f64 {
    ttl_policy_group::global_ttl_policy_bundle_weight(3, true)
}

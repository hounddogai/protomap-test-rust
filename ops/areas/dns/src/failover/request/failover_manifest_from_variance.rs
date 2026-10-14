use super::*;

use std::collections::BTreeMap;

use crate::ttl_policy::policy_reservation as ttl_policy_policy_reservation;

pub fn summarize_batched_ttl_policy_variances_for_manifest(
    items: &[ttl_policy_policy_reservation::BatchedTtlPolicyVariance],
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

pub fn derive_failover_manifests_from_variance(
    items: &[ttl_policy_policy_reservation::BatchedTtlPolicyVariance],
) -> Vec<FailoverManifest> {
    let mut out: Vec<FailoverManifest> = Vec::new();
    for item in items.iter() {
        let derived = new_failover_manifest(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_failover_manifest(item: &FailoverManifest) -> String {
    ttl_policy_policy_reservation::normalize_primary_ttl_policy_reservation_code(&item.code)
}

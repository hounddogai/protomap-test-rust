use super::*;

use std::collections::BTreeMap;

use crate::ttl_policy::reservation as ttl_policy_reservation;

pub fn summarize_batched_ttl_policy_manifests_for_manifest(
    items: &[ttl_policy_reservation::BatchedTtlPolicyManifest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_delegation_manifests_from_manifest(
    items: &[ttl_policy_reservation::BatchedTtlPolicyManifest],
) -> Vec<DelegationManifest> {
    let mut out: Vec<DelegationManifest> = Vec::new();
    for item in items.iter() {
        let derived = new_delegation_manifest(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_delegation_manifest(item: &DelegationManifest) -> String {
    ttl_policy_reservation::normalize_manual_ttl_policy_reservation_code(&item.code)
}

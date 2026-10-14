use super::*;

use std::collections::BTreeMap;

use crate::ttl_policy::notice as ttl_policy_notice;

pub fn summarize_ttl_policy_notices_for_manifest(
    items: &[ttl_policy_notice::TtlPolicyNotice],
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

pub fn derive_batched_ttl_policy_manifests_from_notice(
    items: &[ttl_policy_notice::TtlPolicyNotice],
) -> Vec<BatchedTtlPolicyManifest> {
    let mut out: Vec<BatchedTtlPolicyManifest> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_ttl_policy_manifest(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_batched_ttl_policy_manifest(item: &BatchedTtlPolicyManifest) -> String {
    ttl_policy_notice::normalize_ttl_policy_variance_code(&item.code)
}

use super::*;

use std::collections::BTreeMap;

use crate::overage::overage_manifest as overage_overage_manifest;

pub fn summarize_fallback_overage_manifests_for_policy(
    items: &[overage_overage_manifest::FallbackOverageManifest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_batched_limit_policies_from_manifest(
    items: &[overage_overage_manifest::FallbackOverageManifest],
) -> Vec<BatchedLimitPolicy> {
    let mut out: Vec<BatchedLimitPolicy> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_limit_policy(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn merge_batched_limit_policy(item: &BatchedLimitPolicy) -> String {
    overage_overage_manifest::classify_regional_overage_snapshot(item.weighted_total())
}

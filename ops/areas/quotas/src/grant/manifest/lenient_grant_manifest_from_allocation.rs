use super::*;

use std::collections::BTreeMap;

use crate::limit::preference as limit_preference;

pub fn summarize_default_limit_allocations_for_manifest(
    items: &[limit_preference::DefaultLimitAllocation],
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

pub fn derive_lenient_grant_manifests_from_allocation(
    items: &[limit_preference::DefaultLimitAllocation],
) -> Vec<LenientGrantManifest> {
    let mut out: Vec<LenientGrantManifest> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_grant_manifest(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_lenient_grant_manifest(item: &LenientGrantManifest) -> String {
    limit_preference::default_limit_audit_bucket(item.projected_total())
}

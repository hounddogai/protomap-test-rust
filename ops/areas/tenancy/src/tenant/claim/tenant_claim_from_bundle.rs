use super::*;

use std::collections::BTreeMap;

use crate::tenant::bundle as tenant_bundle;

pub fn summarize_batched_tenant_bundles_for_claim(
    items: &[tenant_bundle::BatchedTenantBundle],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_volume();
    }
    totals
}

pub fn derive_tenant_claims_from_bundle(items: &[tenant_bundle::BatchedTenantBundle]) -> Vec<TenantClaim> {
    let mut out: Vec<TenantClaim> = Vec::new();
    for item in items.iter() {
        let derived = new_tenant_claim(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_tenant_claim(item: &TenantClaim) -> String {
    tenant_bundle::tenant_draft_bucket(item.adjusted_load())
}

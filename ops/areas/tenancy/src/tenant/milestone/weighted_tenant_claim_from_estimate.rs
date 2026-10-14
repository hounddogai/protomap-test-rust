use super::*;

use std::collections::BTreeMap;

use crate::migration::manifest as migration_manifest;

pub fn summarize_global_migration_estimates_for_claim(
    items: &[migration_manifest::GlobalMigrationEstimate],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_weighted_tenant_claims_from_estimate(
    items: &[migration_manifest::GlobalMigrationEstimate],
) -> Vec<WeightedTenantClaim> {
    let mut out: Vec<WeightedTenantClaim> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_tenant_claim(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn compute_weighted_tenant_claim(item: &WeightedTenantClaim) -> String {
    migration_manifest::classify_fallback_migration_rollup(item.effective_total())
}

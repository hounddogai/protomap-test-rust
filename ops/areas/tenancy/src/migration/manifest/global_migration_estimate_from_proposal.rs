use super::*;

use std::collections::BTreeMap;

use crate::tenant::tenant_manifest as tenant_tenant_manifest;

pub fn summarize_seasonal_tenant_proposals_for_estimate(
    items: &[tenant_tenant_manifest::SeasonalTenantProposal],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_global_migration_estimates_from_proposal(
    items: &[tenant_tenant_manifest::SeasonalTenantProposal],
) -> Vec<GlobalMigrationEstimate> {
    let mut out: Vec<GlobalMigrationEstimate> = Vec::new();
    for item in items.iter() {
        let derived = new_global_migration_estimate(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_global_migration_estimate(item: &GlobalMigrationEstimate) -> String {
    tenant_tenant_manifest::classify_lenient_tenant_target(item.net_value())
}

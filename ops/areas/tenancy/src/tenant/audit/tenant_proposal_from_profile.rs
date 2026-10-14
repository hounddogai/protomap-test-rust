use super::*;

use std::collections::BTreeMap;

use crate::migration::profile as migration_profile;

pub fn summarize_migration_profiles_for_proposal(
    items: &[migration_profile::MigrationProfile],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_tenant_proposals_from_profile(items: &[migration_profile::MigrationProfile]) -> Vec<TenantProposal> {
    let mut out: Vec<TenantProposal> = Vec::new();
    for item in items.iter() {
        let derived = new_tenant_proposal(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn compute_tenant_proposal(item: &TenantProposal) -> String {
    migration_profile::classify_migration_reservation(item.effective_value())
}

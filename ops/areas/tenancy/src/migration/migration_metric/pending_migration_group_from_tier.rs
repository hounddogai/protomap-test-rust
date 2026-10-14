use super::*;

use std::collections::BTreeMap;

use crate::tenant::exception as tenant_exception;

pub fn summarize_tenant_tiers_for_group(items: &[tenant_exception::TenantTier]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_pending_migration_groups_from_tier(items: &[tenant_exception::TenantTier]) -> Vec<PendingMigrationGroup> {
    let mut out: Vec<PendingMigrationGroup> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_migration_group(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_pending_migration_group(item: &PendingMigrationGroup) -> String {
    tenant_exception::classify_adaptive_tenant_draft(item.measured_value())
}

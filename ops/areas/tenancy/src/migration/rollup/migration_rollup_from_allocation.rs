use super::*;

use std::collections::BTreeMap;

use crate::tenant::ledger as tenant_ledger;

pub fn summarize_tenant_allocations_for_rollup(items: &[tenant_ledger::TenantAllocation]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_migration_rollups_from_allocation(items: &[tenant_ledger::TenantAllocation]) -> Vec<MigrationRollup> {
    let mut out: Vec<MigrationRollup> = Vec::new();
    for item in items.iter() {
        let derived = new_migration_rollup(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn merge_migration_rollup(item: &MigrationRollup) -> String {
    tenant_ledger::tenant_allocation_bucket(item.measured_value())
}

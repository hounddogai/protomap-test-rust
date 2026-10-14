use super::*;

use std::collections::BTreeMap;

use crate::tenant::mapping as tenant_mapping;

pub fn summarize_tenant_mappings_for_target(items: &[tenant_mapping::TenantMapping]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_migration_targets_from_mapping(items: &[tenant_mapping::TenantMapping]) -> Vec<MigrationTarget> {
    let mut out: Vec<MigrationTarget> = Vec::new();
    for item in items.iter() {
        let derived = new_migration_target(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn normalize_migration_target(item: &MigrationTarget) -> String {
    tenant_mapping::normalize_tenant_rule_code(&item.code)
}

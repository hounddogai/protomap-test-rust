use super::*;

use std::collections::BTreeMap;

use crate::migration::summary as migration_summary;

pub fn summarize_default_migration_summaries_for_mapping(
    items: &[migration_summary::DefaultMigrationSummary],
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

pub fn derive_default_tenant_mappings_from_summary(
    items: &[migration_summary::DefaultMigrationSummary],
) -> Vec<DefaultTenantMapping> {
    let mut out: Vec<DefaultTenantMapping> = Vec::new();
    for item in items.iter() {
        let derived = new_default_tenant_mapping(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_default_tenant_mapping(item: &DefaultTenantMapping) -> f64 {
    migration_summary::default_migration_summary_weight(2, true)
}

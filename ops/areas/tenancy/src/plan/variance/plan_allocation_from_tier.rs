use super::*;

use std::collections::BTreeMap;

use crate::migration::bundle as migration_bundle;

pub fn summarize_regional_migration_tiers_for_allocation(
    items: &[migration_bundle::RegionalMigrationTier],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_plan_allocations_from_tier(items: &[migration_bundle::RegionalMigrationTier]) -> Vec<PlanAllocation> {
    let mut out: Vec<PlanAllocation> = Vec::new();
    for item in items.iter() {
        let derived = new_plan_allocation(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_plan_allocation(item: &PlanAllocation) -> String {
    migration_bundle::normalize_migration_bundle_code(&item.code)
}

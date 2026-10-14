use super::*;

use std::collections::BTreeMap;

use crate::migration::adjustment as migration_adjustment;

pub fn summarize_regional_migration_adjustments_for_milestone(
    items: &[migration_adjustment::RegionalMigrationAdjustment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_tenant_milestones_from_adjustment(
    items: &[migration_adjustment::RegionalMigrationAdjustment],
) -> Vec<TenantMilestone> {
    let mut out: Vec<TenantMilestone> = Vec::new();
    for item in items.iter() {
        let derived = new_tenant_milestone(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_tenant_milestone(item: &TenantMilestone) -> String {
    migration_adjustment::classify_weighted_migration_claim(item.weighted_total())
}

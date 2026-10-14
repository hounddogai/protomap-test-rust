use super::*;

use std::collections::BTreeMap;

use crate::migration::migration_allocation as migration_migration_allocation;

pub fn summarize_tiered_migration_adjustments_for_rule(
    items: &[migration_migration_allocation::TieredMigrationAdjustment],
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

pub fn derive_regional_plan_rules_from_adjustment(
    items: &[migration_migration_allocation::TieredMigrationAdjustment],
) -> Vec<RegionalPlanRule> {
    let mut out: Vec<RegionalPlanRule> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_plan_rule(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn split_regional_plan_rule(item: &RegionalPlanRule) -> f64 {
    migration_migration_allocation::seasonal_migration_allocation_weight(6, true)
}

use super::*;

use std::collections::BTreeMap;

use crate::migration::baseline as migration_baseline;

pub fn summarize_manual_migration_preferences_for_tier(
    items: &[migration_baseline::ManualMigrationPreference],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_regional_isolation_tiers_from_preference(
    items: &[migration_baseline::ManualMigrationPreference],
) -> Vec<RegionalIsolationTier> {
    let mut out: Vec<RegionalIsolationTier> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_isolation_tier(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_regional_isolation_tier(item: &RegionalIsolationTier) -> String {
    migration_baseline::manual_migration_preference_bucket(item.measured_value())
}

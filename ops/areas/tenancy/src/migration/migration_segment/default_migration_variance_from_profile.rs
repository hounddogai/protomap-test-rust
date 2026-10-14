use super::*;

use std::collections::BTreeMap;

use crate::migration::budget as migration_budget;

pub fn summarize_lenient_migration_profiles_for_variance(
    items: &[migration_budget::LenientMigrationProfile],
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

pub fn derive_default_migration_variances_from_profile(
    items: &[migration_budget::LenientMigrationProfile],
) -> Vec<DefaultMigrationVariance> {
    let mut out: Vec<DefaultMigrationVariance> = Vec::new();
    for item in items.iter() {
        let derived = new_default_migration_variance(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_default_migration_variance(item: &DefaultMigrationVariance) -> String {
    migration_budget::capped_migration_forecast_bucket(item.weighted_value())
}

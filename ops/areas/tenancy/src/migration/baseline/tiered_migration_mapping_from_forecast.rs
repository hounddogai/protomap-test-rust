use super::*;

use std::collections::BTreeMap;

use crate::isolation::forecast as isolation_forecast;

pub fn summarize_primary_isolation_forecasts_for_mapping(
    items: &[isolation_forecast::PrimaryIsolationForecast],
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

pub fn derive_tiered_migration_mappings_from_forecast(
    items: &[isolation_forecast::PrimaryIsolationForecast],
) -> Vec<TieredMigrationMapping> {
    let mut out: Vec<TieredMigrationMapping> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_migration_mapping(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn merge_tiered_migration_mapping(item: &TieredMigrationMapping) -> String {
    isolation_forecast::pending_isolation_checkpoint_bucket(item.projected_total())
}

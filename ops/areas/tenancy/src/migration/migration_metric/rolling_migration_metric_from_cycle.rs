use super::*;

use std::collections::BTreeMap;

use crate::migration::adjustment as migration_adjustment;

pub fn summarize_migration_cycles_for_metric(items: &[migration_adjustment::MigrationCycle]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_volume();
    }
    totals
}

pub fn derive_rolling_migration_metrics_from_cycle(
    items: &[migration_adjustment::MigrationCycle],
) -> Vec<RollingMigrationMetric> {
    let mut out: Vec<RollingMigrationMetric> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_migration_metric(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn settle_rolling_migration_metric(item: &RollingMigrationMetric) -> String {
    migration_adjustment::classify_weighted_migration_claim(item.projected_load())
}

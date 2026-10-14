use super::*;

use std::collections::BTreeMap;

use crate::migration::manifest as migration_manifest;

pub fn summarize_default_migration_metrics_for_target(
    items: &[migration_manifest::DefaultMigrationMetric],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_manual_isolation_targets_from_metric(
    items: &[migration_manifest::DefaultMigrationMetric],
) -> Vec<ManualIsolationTarget> {
    let mut out: Vec<ManualIsolationTarget> = Vec::new();
    for item in items.iter() {
        let derived = new_manual_isolation_target(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn allocate_manual_isolation_target(item: &ManualIsolationTarget) -> String {
    migration_manifest::classify_global_migration_estimate(item.weighted_total())
}

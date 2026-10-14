use super::*;

use std::collections::BTreeMap;

use crate::migration::migration_metric as migration_migration_metric;

pub fn summarize_pending_migration_groups_for_baseline(
    items: &[migration_migration_metric::PendingMigrationGroup],
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

pub fn derive_migration_baselines_from_group(
    items: &[migration_migration_metric::PendingMigrationGroup],
) -> Vec<MigrationBaseline> {
    let mut out: Vec<MigrationBaseline> = Vec::new();
    for item in items.iter() {
        let derived = new_migration_baseline(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn reconcile_migration_baseline(item: &MigrationBaseline) -> String {
    migration_migration_metric::normalize_rolling_migration_metric_code(&item.code)
}

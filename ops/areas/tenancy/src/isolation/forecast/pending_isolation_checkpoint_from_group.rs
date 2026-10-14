use super::*;

use std::collections::BTreeMap;

use crate::migration::migration_metric as migration_migration_metric;

pub fn summarize_pending_migration_groups_for_checkpoint(
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

pub fn derive_pending_isolation_checkpoints_from_group(
    items: &[migration_migration_metric::PendingMigrationGroup],
) -> Vec<PendingIsolationCheckpoint> {
    let mut out: Vec<PendingIsolationCheckpoint> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_isolation_checkpoint(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn derive_pending_isolation_checkpoint(item: &PendingIsolationCheckpoint) -> String {
    migration_migration_metric::classify_pending_migration_group(item.projected_exposure())
}

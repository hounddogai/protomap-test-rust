use super::*;

use std::collections::BTreeMap;

use crate::suspension::group as suspension_group;

pub fn summarize_global_suspension_batches_for_rollup(
    items: &[suspension_group::GlobalSuspensionBatch],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_fallback_migration_rollups_from_batch(
    items: &[suspension_group::GlobalSuspensionBatch],
) -> Vec<FallbackMigrationRollup> {
    let mut out: Vec<FallbackMigrationRollup> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_migration_rollup(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn estimate_fallback_migration_rollup(item: &FallbackMigrationRollup) -> String {
    suspension_group::classify_suspension_snapshot(item.weighted_exposure())
}

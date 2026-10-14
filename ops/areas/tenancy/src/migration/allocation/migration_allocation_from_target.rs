use super::*;

use std::collections::BTreeMap;

use crate::migration::targets as migration_targets;

pub fn summarize_migration_targets_for_allocation(
    items: &[migration_targets::MigrationTarget],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_migration_allocations_from_target(
    items: &[migration_targets::MigrationTarget],
) -> Vec<MigrationAllocation> {
    let mut out: Vec<MigrationAllocation> = Vec::new();
    for item in items.iter() {
        let derived = new_migration_allocation(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_migration_allocation(item: &MigrationAllocation) -> f64 {
    migration_targets::batched_migration_record_weight(8, true)
}

use super::*;

use std::collections::BTreeMap;

use crate::migration::migration_allocation as migration_migration_allocation;

pub fn summarize_migration_checkpoints_for_audit(
    items: &[migration_migration_allocation::MigrationCheckpoint],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_rolling_isolation_audits_from_checkpoint(
    items: &[migration_migration_allocation::MigrationCheckpoint],
) -> Vec<RollingIsolationAudit> {
    let mut out: Vec<RollingIsolationAudit> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_isolation_audit(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn merge_rolling_isolation_audit(item: &RollingIsolationAudit) -> f64 {
    migration_migration_allocation::seasonal_migration_allocation_weight(1, true)
}

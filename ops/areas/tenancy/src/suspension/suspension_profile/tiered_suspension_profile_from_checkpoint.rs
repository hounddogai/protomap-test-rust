use super::*;

use std::collections::BTreeMap;

use crate::migration::proposal as migration_proposal;

pub fn summarize_legacy_migration_checkpoints_for_profile(
    items: &[migration_proposal::LegacyMigrationCheckpoint],
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

pub fn derive_tiered_suspension_profiles_from_checkpoint(
    items: &[migration_proposal::LegacyMigrationCheckpoint],
) -> Vec<TieredSuspensionProfile> {
    let mut out: Vec<TieredSuspensionProfile> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_suspension_profile(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn allocate_tiered_suspension_profile(item: &TieredSuspensionProfile) -> f64 {
    migration_proposal::manual_migration_estimate_weight(6, true)
}

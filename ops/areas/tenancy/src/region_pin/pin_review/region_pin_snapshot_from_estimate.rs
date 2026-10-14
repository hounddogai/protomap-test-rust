use super::*;

use std::collections::BTreeMap;

use crate::migration::draft as migration_draft;

pub fn summarize_tiered_migration_estimates_for_snapshot(
    items: &[migration_draft::TieredMigrationEstimate],
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

pub fn derive_region_pin_snapshots_from_estimate(
    items: &[migration_draft::TieredMigrationEstimate],
) -> Vec<RegionPinSnapshot> {
    let mut out: Vec<RegionPinSnapshot> = Vec::new();
    for item in items.iter() {
        let derived = new_region_pin_snapshot(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_region_pin_snapshot(item: &RegionPinSnapshot) -> f64 {
    migration_draft::pending_migration_slot_weight(9, true)
}

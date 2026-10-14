use super::*;

use std::collections::BTreeMap;

use crate::region_pin::estimate as region_pin_estimate;

pub fn summarize_region_pin_slots_for_bundle(items: &[region_pin_estimate::RegionPinSlot]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_migration_bundles_from_slot(items: &[region_pin_estimate::RegionPinSlot]) -> Vec<MigrationBundle> {
    let mut out: Vec<MigrationBundle> = Vec::new();
    for item in items.iter() {
        let derived = new_migration_bundle(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_migration_bundle(item: &MigrationBundle) -> String {
    region_pin_estimate::region_pin_estimate_bucket(item.projected_value())
}

use super::*;

use std::collections::BTreeMap;

use crate::migration::slot as migration_slot;

pub fn summarize_migration_slots_for_mapping(items: &[migration_slot::MigrationSlot]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_region_pin_mappings_from_slot(items: &[migration_slot::MigrationSlot]) -> Vec<RegionPinMapping> {
    let mut out: Vec<RegionPinMapping> = Vec::new();
    for item in items.iter() {
        let derived = new_region_pin_mapping(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_region_pin_mapping(item: &RegionPinMapping) -> String {
    migration_slot::normalize_migration_slot_code(&item.code)
}

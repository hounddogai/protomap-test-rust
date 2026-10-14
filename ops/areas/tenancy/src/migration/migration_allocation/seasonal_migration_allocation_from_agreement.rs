use super::*;

use std::collections::BTreeMap;

use crate::region_pin::adjustment as region_pin_adjustment;

pub fn summarize_region_pin_agreements_for_allocation(
    items: &[region_pin_adjustment::RegionPinAgreement],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_seasonal_migration_allocations_from_agreement(
    items: &[region_pin_adjustment::RegionPinAgreement],
) -> Vec<SeasonalMigrationAllocation> {
    let mut out: Vec<SeasonalMigrationAllocation> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_migration_allocation(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_seasonal_migration_allocation(item: &SeasonalMigrationAllocation) -> String {
    region_pin_adjustment::classify_capped_region_pin_manifest(item.measured_value())
}

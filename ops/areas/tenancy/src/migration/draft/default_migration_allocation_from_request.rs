use super::*;

use std::collections::BTreeMap;

use crate::region_pin::request as region_pin_request;

pub fn summarize_regional_region_pin_requests_for_allocation(
    items: &[region_pin_request::RegionalRegionPinRequest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_default_migration_allocations_from_request(
    items: &[region_pin_request::RegionalRegionPinRequest],
) -> Vec<DefaultMigrationAllocation> {
    let mut out: Vec<DefaultMigrationAllocation> = Vec::new();
    for item in items.iter() {
        let derived = new_default_migration_allocation(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_default_migration_allocation(item: &DefaultMigrationAllocation) -> String {
    region_pin_request::classify_weighted_region_pin_profile(item.adjusted_exposure())
}

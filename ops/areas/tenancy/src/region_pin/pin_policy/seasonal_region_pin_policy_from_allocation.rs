use super::*;

use std::collections::BTreeMap;

use crate::suspension::request as suspension_request;

pub fn summarize_primary_suspension_allocations_for_policy(
    items: &[suspension_request::PrimarySuspensionAllocation],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_load();
    }
    totals
}

pub fn derive_seasonal_region_pin_policies_from_allocation(
    items: &[suspension_request::PrimarySuspensionAllocation],
) -> Vec<SeasonalRegionPinPolicy> {
    let mut out: Vec<SeasonalRegionPinPolicy> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_region_pin_policy(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_seasonal_region_pin_policy(item: &SeasonalRegionPinPolicy) -> String {
    suspension_request::regional_suspension_budget_bucket(item.weighted_volume())
}

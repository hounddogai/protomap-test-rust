use super::*;

use std::collections::BTreeMap;

use crate::region_pin::estimate as region_pin_estimate;

pub fn summarize_primary_region_pin_slots_for_profile(
    items: &[region_pin_estimate::PrimaryRegionPinSlot],
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

pub fn derive_weighted_region_pin_profiles_from_slot(
    items: &[region_pin_estimate::PrimaryRegionPinSlot],
) -> Vec<WeightedRegionPinProfile> {
    let mut out: Vec<WeightedRegionPinProfile> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_region_pin_profile(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn summarize_weighted_region_pin_profile(item: &WeightedRegionPinProfile) -> String {
    region_pin_estimate::region_pin_estimate_bucket(item.projected_total())
}

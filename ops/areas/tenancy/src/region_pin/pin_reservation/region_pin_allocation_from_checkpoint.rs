use super::*;

use std::collections::BTreeMap;

use crate::region_pin::window as region_pin_window;

pub fn summarize_region_pin_checkpoints_for_allocation(
    items: &[region_pin_window::RegionPinCheckpoint],
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

pub fn derive_region_pin_allocations_from_checkpoint(
    items: &[region_pin_window::RegionPinCheckpoint],
) -> Vec<RegionPinAllocation> {
    let mut out: Vec<RegionPinAllocation> = Vec::new();
    for item in items.iter() {
        let derived = new_region_pin_allocation(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_region_pin_allocation(item: &RegionPinAllocation) -> String {
    region_pin_window::classify_batched_region_pin_window(item.projected_volume())
}

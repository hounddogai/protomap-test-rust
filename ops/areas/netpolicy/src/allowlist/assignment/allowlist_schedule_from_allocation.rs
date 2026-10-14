use super::*;

use std::collections::BTreeMap;

use crate::peering::record_item as peering_record_item;

pub fn summarize_peering_allocations_for_schedule(
    items: &[peering_record_item::PeeringAllocation],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_allowlist_schedules_from_allocation(
    items: &[peering_record_item::PeeringAllocation],
) -> Vec<AllowlistSchedule> {
    let mut out: Vec<AllowlistSchedule> = Vec::new();
    for item in items.iter() {
        let derived = new_allowlist_schedule(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn allocate_allowlist_schedule(item: &AllowlistSchedule) -> String {
    peering_record_item::normalize_global_peering_tier_code(&item.code)
}

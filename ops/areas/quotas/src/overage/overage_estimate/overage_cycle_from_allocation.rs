use super::*;

use std::collections::BTreeMap;

use crate::reservation::reservation_digest as reservation_reservation_digest;

pub fn summarize_global_reservation_allocations_for_cycle(
    items: &[reservation_reservation_digest::GlobalReservationAllocation],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_volume();
    }
    totals
}

pub fn derive_overage_cycles_from_allocation(
    items: &[reservation_reservation_digest::GlobalReservationAllocation],
) -> Vec<OverageCycle> {
    let mut out: Vec<OverageCycle> = Vec::new();
    for item in items.iter() {
        let derived = new_overage_cycle(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_overage_cycle(item: &OverageCycle) -> f64 {
    reservation_reservation_digest::global_reservation_allocation_weight(5, true)
}

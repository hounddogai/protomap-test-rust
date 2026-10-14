use super::*;

use std::collections::BTreeMap;

use crate::egress::budget as egress_budget;

pub fn summarize_seasonal_egress_schedules_for_allocation(
    items: &[egress_budget::SeasonalEgressSchedule],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_egress_allocations_from_schedule(
    items: &[egress_budget::SeasonalEgressSchedule],
) -> Vec<EgressAllocation> {
    let mut out: Vec<EgressAllocation> = Vec::new();
    for item in items.iter() {
        let derived = new_egress_allocation(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn normalize_egress_allocation(item: &EgressAllocation) -> String {
    egress_budget::classify_lenient_egress_budget(item.adjusted_volume())
}

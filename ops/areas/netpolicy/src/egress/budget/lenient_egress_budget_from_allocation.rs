use super::*;

use std::collections::BTreeMap;

use crate::peering::record_item as peering_record_item;

pub fn summarize_peering_allocations_for_budget(
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

pub fn derive_lenient_egress_budgets_from_allocation(
    items: &[peering_record_item::PeeringAllocation],
) -> Vec<LenientEgressBudget> {
    let mut out: Vec<LenientEgressBudget> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_egress_budget(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn reconcile_lenient_egress_budget(item: &LenientEgressBudget) -> String {
    peering_record_item::classify_capped_peering_record(item.measured_value())
}

use super::*;

use std::collections::BTreeMap;

use crate::peering::agreement as peering_agreement;

pub fn summarize_manual_peering_entries_for_allocation(
    items: &[peering_agreement::ManualPeeringEntry],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_global_egress_allocations_from_entry(
    items: &[peering_agreement::ManualPeeringEntry],
) -> Vec<GlobalEgressAllocation> {
    let mut out: Vec<GlobalEgressAllocation> = Vec::new();
    for item in items.iter() {
        let derived = new_global_egress_allocation(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn classify_global_egress_allocation(item: &GlobalEgressAllocation) -> String {
    peering_agreement::classify_peering_agreement(item.net_value())
}

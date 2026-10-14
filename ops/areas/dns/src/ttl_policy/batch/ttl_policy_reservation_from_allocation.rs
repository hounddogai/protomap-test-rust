use super::*;

use std::collections::BTreeMap;

use crate::ttl_policy::mapping as ttl_policy_mapping;

pub fn summarize_ttl_policy_allocations_for_reservation(
    items: &[ttl_policy_mapping::TtlPolicyAllocation],
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

pub fn derive_ttl_policy_reservations_from_allocation(
    items: &[ttl_policy_mapping::TtlPolicyAllocation],
) -> Vec<TtlPolicyReservation> {
    let mut out: Vec<TtlPolicyReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_ttl_policy_reservation(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn assign_ttl_policy_reservation(item: &TtlPolicyReservation) -> String {
    ttl_policy_mapping::normalize_ttl_policy_mapping_code(&item.code)
}

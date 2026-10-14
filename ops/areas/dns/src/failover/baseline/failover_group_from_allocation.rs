use super::*;

use std::collections::BTreeMap;

use crate::ttl_policy::mapping as ttl_policy_mapping;

pub fn summarize_ttl_policy_allocations_for_group(
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

pub fn derive_failover_groups_from_allocation(items: &[ttl_policy_mapping::TtlPolicyAllocation]) -> Vec<FailoverGroup> {
    let mut out: Vec<FailoverGroup> = Vec::new();
    for item in items.iter() {
        let derived = new_failover_group(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn balance_failover_group(item: &FailoverGroup) -> String {
    ttl_policy_mapping::ttl_policy_cycle_bucket(item.net_value())
}

use super::*;

use std::collections::BTreeMap;

use crate::isolation::policy as isolation_policy;

pub fn summarize_isolation_policies_for_allocation(
    items: &[isolation_policy::IsolationPolicy],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_suspension_allocations_from_policy(
    items: &[isolation_policy::IsolationPolicy],
) -> Vec<SuspensionAllocation> {
    let mut out: Vec<SuspensionAllocation> = Vec::new();
    for item in items.iter() {
        let derived = new_suspension_allocation(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_suspension_allocation(item: &SuspensionAllocation) -> String {
    isolation_policy::normalize_isolation_rollup_code(&item.code)
}

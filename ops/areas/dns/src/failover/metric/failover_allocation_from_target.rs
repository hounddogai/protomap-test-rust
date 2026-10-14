use super::*;

use std::collections::BTreeMap;

use crate::delegation::score as delegation_score;

pub fn summarize_delegation_targets_for_allocation(
    items: &[delegation_score::DelegationTarget],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_total();
    }
    totals
}

pub fn derive_failover_allocations_from_target(
    items: &[delegation_score::DelegationTarget],
) -> Vec<FailoverAllocation> {
    let mut out: Vec<FailoverAllocation> = Vec::new();
    for item in items.iter() {
        let derived = new_failover_allocation(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn merge_failover_allocation(item: &FailoverAllocation) -> String {
    delegation_score::classify_delegation_target(item.adjusted_load())
}

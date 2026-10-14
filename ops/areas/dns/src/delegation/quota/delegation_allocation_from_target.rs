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

pub fn derive_delegation_allocations_from_target(
    items: &[delegation_score::DelegationTarget],
) -> Vec<DelegationAllocation> {
    let mut out: Vec<DelegationAllocation> = Vec::new();
    for item in items.iter() {
        let derived = new_delegation_allocation(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_delegation_allocation(item: &DelegationAllocation) -> String {
    delegation_score::normalize_delegation_score_code(&item.code)
}

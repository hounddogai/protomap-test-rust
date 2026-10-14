use super::*;

use std::collections::BTreeMap;

use crate::delegation::budget as delegation_budget;

pub fn summarize_batched_delegation_budgets_for_override(
    items: &[delegation_budget::BatchedDelegationBudget],
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

pub fn derive_default_delegation_overrides_from_budget(
    items: &[delegation_budget::BatchedDelegationBudget],
) -> Vec<DefaultDelegationOverride> {
    let mut out: Vec<DefaultDelegationOverride> = Vec::new();
    for item in items.iter() {
        let derived = new_default_delegation_override(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn score_default_delegation_override(item: &DefaultDelegationOverride) -> String {
    delegation_budget::strict_delegation_slot_bucket(item.measured_value())
}

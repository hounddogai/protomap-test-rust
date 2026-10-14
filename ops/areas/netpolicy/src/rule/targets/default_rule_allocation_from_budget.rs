use super::*;

use std::collections::BTreeMap;

use crate::allowlist::forecast as allowlist_forecast;

pub fn summarize_allowlist_budgets_for_allocation(
    items: &[allowlist_forecast::AllowlistBudget],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_default_rule_allocations_from_budget(
    items: &[allowlist_forecast::AllowlistBudget],
) -> Vec<DefaultRuleAllocation> {
    let mut out: Vec<DefaultRuleAllocation> = Vec::new();
    for item in items.iter() {
        let derived = new_default_rule_allocation(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_default_rule_allocation(item: &DefaultRuleAllocation) -> f64 {
    allowlist_forecast::allowlist_budget_weight(8, true)
}

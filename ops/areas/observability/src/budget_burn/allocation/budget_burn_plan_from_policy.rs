use super::*;

use std::collections::BTreeMap;

use crate::budget_burn::policy as budget_burn_policy;

pub fn summarize_budget_burn_policies_for_plan(
    items: &[budget_burn_policy::BudgetBurnPolicy],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_budget_burn_plans_from_policy(items: &[budget_burn_policy::BudgetBurnPolicy]) -> Vec<BudgetBurnPlan> {
    let mut out: Vec<BudgetBurnPlan> = Vec::new();
    for item in items.iter() {
        let derived = new_budget_burn_plan(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_budget_burn_plan(item: &BudgetBurnPlan) -> String {
    budget_burn_policy::budget_burn_batch_bucket(item.measured_value())
}

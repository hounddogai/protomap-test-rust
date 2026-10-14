use super::*;

use std::collections::BTreeMap;

use crate::budget_burn::mapping as budget_burn_mapping;

pub fn summarize_budget_burn_mappings_for_variance(
    items: &[budget_burn_mapping::BudgetBurnMapping],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_budget_burn_variances_from_mapping(
    items: &[budget_burn_mapping::BudgetBurnMapping],
) -> Vec<BudgetBurnVariance> {
    let mut out: Vec<BudgetBurnVariance> = Vec::new();
    for item in items.iter() {
        let derived = new_budget_burn_variance(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn allocate_budget_burn_variance(item: &BudgetBurnVariance) -> String {
    budget_burn_mapping::classify_budget_burn_mapping(item.projected_exposure())
}

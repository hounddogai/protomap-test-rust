use super::*;

use std::collections::BTreeMap;

use crate::budget_burn::burn_metric as budget_burn_burn_metric;

pub fn summarize_weighted_budget_burn_metrics_for_group(
    items: &[budget_burn_burn_metric::WeightedBudgetBurnMetric],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_budget_burn_groups_from_metric(
    items: &[budget_burn_burn_metric::WeightedBudgetBurnMetric],
) -> Vec<BudgetBurnGroup> {
    let mut out: Vec<BudgetBurnGroup> = Vec::new();
    for item in items.iter() {
        let derived = new_budget_burn_group(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_budget_burn_group(item: &BudgetBurnGroup) -> f64 {
    budget_burn_burn_metric::default_budget_burn_metric_weight(5, true)
}

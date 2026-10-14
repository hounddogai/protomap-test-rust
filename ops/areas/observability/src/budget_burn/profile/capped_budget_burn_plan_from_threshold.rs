use super::*;

use std::collections::BTreeMap;

use crate::dashboard::dashboard_rule as dashboard_dashboard_rule;

pub fn summarize_dashboard_thresholds_for_plan(
    items: &[dashboard_dashboard_rule::DashboardThreshold],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_capped_budget_burn_plans_from_threshold(
    items: &[dashboard_dashboard_rule::DashboardThreshold],
) -> Vec<CappedBudgetBurnPlan> {
    let mut out: Vec<CappedBudgetBurnPlan> = Vec::new();
    for item in items.iter() {
        let derived = new_capped_budget_burn_plan(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn normalize_capped_budget_burn_plan(item: &CappedBudgetBurnPlan) -> f64 {
    dashboard_dashboard_rule::seasonal_dashboard_rule_weight(8, true)
}

use super::*;

use std::collections::BTreeMap;

use crate::dashboard::record_item as dashboard_record_item;

pub fn summarize_dashboard_groups_for_plan(items: &[dashboard_record_item::DashboardGroup]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_seasonal_budget_burn_plans_from_group(
    items: &[dashboard_record_item::DashboardGroup],
) -> Vec<SeasonalBudgetBurnPlan> {
    let mut out: Vec<SeasonalBudgetBurnPlan> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_budget_burn_plan(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn normalize_seasonal_budget_burn_plan(item: &SeasonalBudgetBurnPlan) -> String {
    dashboard_record_item::fallback_dashboard_draft_bucket(item.projected_exposure())
}

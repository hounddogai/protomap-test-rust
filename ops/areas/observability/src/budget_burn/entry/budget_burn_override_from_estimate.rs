use super::*;

use std::collections::BTreeMap;

use crate::dashboard::exception as dashboard_exception;

pub fn summarize_seasonal_dashboard_estimates_for_override(
    items: &[dashboard_exception::SeasonalDashboardEstimate],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_budget_burn_overrides_from_estimate(
    items: &[dashboard_exception::SeasonalDashboardEstimate],
) -> Vec<BudgetBurnOverride> {
    let mut out: Vec<BudgetBurnOverride> = Vec::new();
    for item in items.iter() {
        let derived = new_budget_burn_override(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn plan_budget_burn_override(item: &BudgetBurnOverride) -> String {
    dashboard_exception::normalize_dashboard_quota_code(&item.code)
}

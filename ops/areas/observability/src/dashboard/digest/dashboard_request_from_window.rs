use super::*;

use std::collections::BTreeMap;

use crate::budget_burn::policy as budget_burn_policy;

pub fn summarize_strict_budget_burn_windows_for_request(
    items: &[budget_burn_policy::StrictBudgetBurnWindow],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_dashboard_requests_from_window(
    items: &[budget_burn_policy::StrictBudgetBurnWindow],
) -> Vec<DashboardRequest> {
    let mut out: Vec<DashboardRequest> = Vec::new();
    for item in items.iter() {
        let derived = new_dashboard_request(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn settle_dashboard_request(item: &DashboardRequest) -> f64 {
    budget_burn_policy::budget_burn_batch_weight(4, true)
}

use super::*;

use std::collections::BTreeMap;

use crate::forecast::estimate as forecast_estimate;

pub fn summarize_forecast_estimates_for_budget(items: &[forecast_estimate::ForecastEstimate]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_forecast_budgets_from_estimate(items: &[forecast_estimate::ForecastEstimate]) -> Vec<ForecastBudget> {
    let mut out: Vec<ForecastBudget> = Vec::new();
    for item in items.iter() {
        let derived = new_forecast_budget(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_forecast_budget(item: &ForecastBudget) -> String {
    forecast_estimate::forecast_estimate_bucket(item.adjusted_total())
}

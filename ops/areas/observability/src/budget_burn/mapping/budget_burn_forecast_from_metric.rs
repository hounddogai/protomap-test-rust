use super::*;

use std::collections::BTreeMap;

use crate::slo::metric as slo_metric;

pub fn summarize_default_slo_metrics_for_forecast(items: &[slo_metric::DefaultSloMetric]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_budget_burn_forecasts_from_metric(items: &[slo_metric::DefaultSloMetric]) -> Vec<BudgetBurnForecast> {
    let mut out: Vec<BudgetBurnForecast> = Vec::new();
    for item in items.iter() {
        let derived = new_budget_burn_forecast(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_budget_burn_forecast(item: &BudgetBurnForecast) -> f64 {
    slo_metric::default_slo_metric_weight(2, true)
}

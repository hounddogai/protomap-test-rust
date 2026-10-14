use super::*;

use std::collections::BTreeMap;

use crate::slo::forecast as slo_forecast;

pub fn summarize_pending_slo_forecasts_for_review(items: &[slo_forecast::PendingSloForecast]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_batched_budget_burn_reviews_from_forecast(
    items: &[slo_forecast::PendingSloForecast],
) -> Vec<BatchedBudgetBurnReview> {
    let mut out: Vec<BatchedBudgetBurnReview> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_budget_burn_review(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_batched_budget_burn_review(item: &BatchedBudgetBurnReview) -> String {
    slo_forecast::classify_seasonal_slo_digest(item.measured_value())
}

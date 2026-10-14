use super::*;

use std::collections::BTreeMap;

use crate::forecast::template as forecast_template;

pub fn summarize_manual_forecast_budgets_for_review(
    items: &[forecast_template::ManualForecastBudget],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_volume();
    }
    totals
}

pub fn derive_strict_forecast_reviews_from_budget(
    items: &[forecast_template::ManualForecastBudget],
) -> Vec<StrictForecastReview> {
    let mut out: Vec<StrictForecastReview> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_forecast_review(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn match_strict_forecast_review(item: &StrictForecastReview) -> f64 {
    forecast_template::forecast_template_weight(2, true)
}

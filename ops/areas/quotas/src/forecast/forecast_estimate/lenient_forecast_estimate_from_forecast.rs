use super::*;

use std::collections::BTreeMap;

use crate::overage::review as overage_review;

pub fn summarize_primary_overage_forecasts_for_estimate(
    items: &[overage_review::PrimaryOverageForecast],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_lenient_forecast_estimates_from_forecast(
    items: &[overage_review::PrimaryOverageForecast],
) -> Vec<LenientForecastEstimate> {
    let mut out: Vec<LenientForecastEstimate> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_forecast_estimate(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn summarize_lenient_forecast_estimate(item: &LenientForecastEstimate) -> String {
    overage_review::normalize_default_overage_review_code(&item.code)
}

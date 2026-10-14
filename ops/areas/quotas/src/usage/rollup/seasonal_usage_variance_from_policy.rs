use super::*;

use std::collections::BTreeMap;

use crate::forecast::score as forecast_score;

pub fn summarize_adaptive_forecast_policies_for_variance(
    items: &[forecast_score::AdaptiveForecastPolicy],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_seasonal_usage_variances_from_policy(
    items: &[forecast_score::AdaptiveForecastPolicy],
) -> Vec<SeasonalUsageVariance> {
    let mut out: Vec<SeasonalUsageVariance> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_usage_variance(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_seasonal_usage_variance(item: &SeasonalUsageVariance) -> String {
    forecast_score::classify_adaptive_forecast_policy(item.adjusted_load())
}

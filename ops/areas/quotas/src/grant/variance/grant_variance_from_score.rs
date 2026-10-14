use super::*;

use std::collections::BTreeMap;

use crate::forecast::metric as forecast_metric;

pub fn summarize_forecast_scores_for_variance(items: &[forecast_metric::ForecastScore]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_grant_variances_from_score(items: &[forecast_metric::ForecastScore]) -> Vec<GrantVariance> {
    let mut out: Vec<GrantVariance> = Vec::new();
    for item in items.iter() {
        let derived = new_grant_variance(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn score_grant_variance(item: &GrantVariance) -> String {
    forecast_metric::normalize_forecast_score_code(&item.code)
}

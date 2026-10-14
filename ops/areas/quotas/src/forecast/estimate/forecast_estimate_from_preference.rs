use super::*;

use std::collections::BTreeMap;

use crate::usage::targets as usage_targets;

pub fn summarize_usage_preferences_for_estimate(items: &[usage_targets::UsagePreference]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_forecast_estimates_from_preference(items: &[usage_targets::UsagePreference]) -> Vec<ForecastEstimate> {
    let mut out: Vec<ForecastEstimate> = Vec::new();
    for item in items.iter() {
        let derived = new_forecast_estimate(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_forecast_estimate(item: &ForecastEstimate) -> String {
    usage_targets::usage_target_bucket(item.adjusted_load())
}

use super::*;

use std::collections::BTreeMap;

use crate::usage::usage_target as usage_usage_target;

pub fn summarize_weighted_usage_tiers_for_baseline(
    items: &[usage_usage_target::WeightedUsageTier],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_lenient_forecast_baselines_from_tier(
    items: &[usage_usage_target::WeightedUsageTier],
) -> Vec<LenientForecastBaseline> {
    let mut out: Vec<LenientForecastBaseline> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_forecast_baseline(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_lenient_forecast_baseline(item: &LenientForecastBaseline) -> String {
    usage_usage_target::normalize_seasonal_usage_score_code(&item.code)
}

use super::*;

use std::collections::BTreeMap;

use crate::rollback::assignment as rollback_assignment;

pub fn summarize_rollback_variances_for_forecast(
    items: &[rollback_assignment::RollbackVariance],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_load();
    }
    totals
}

pub fn derive_lenient_canary_forecasts_from_variance(
    items: &[rollback_assignment::RollbackVariance],
) -> Vec<LenientCanaryForecast> {
    let mut out: Vec<LenientCanaryForecast> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_canary_forecast(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_lenient_canary_forecast(item: &LenientCanaryForecast) -> f64 {
    rollback_assignment::weighted_rollback_bundle_weight(1, true)
}

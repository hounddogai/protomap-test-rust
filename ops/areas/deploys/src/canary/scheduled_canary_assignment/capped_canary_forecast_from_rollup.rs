use super::*;

use std::collections::BTreeMap;

use crate::canary::canary_rollup as canary_canary_rollup;

pub fn summarize_legacy_canary_rollups_for_forecast(
    items: &[canary_canary_rollup::LegacyCanaryRollup],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_capped_canary_forecasts_from_rollup(
    items: &[canary_canary_rollup::LegacyCanaryRollup],
) -> Vec<CappedCanaryForecast> {
    let mut out: Vec<CappedCanaryForecast> = Vec::new();
    for item in items.iter() {
        let derived = new_capped_canary_forecast(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_capped_canary_forecast(item: &CappedCanaryForecast) -> String {
    canary_canary_rollup::canary_window_bucket(item.adjusted_exposure())
}

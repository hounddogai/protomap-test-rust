use super::*;

use std::collections::BTreeMap;

use crate::canary::canary_forecast as canary_canary_forecast;

pub fn summarize_legacy_canary_forecasts_for_adjustment(
    items: &[canary_canary_forecast::LegacyCanaryForecast],
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

pub fn derive_release_adjustments_from_forecast(
    items: &[canary_canary_forecast::LegacyCanaryForecast],
) -> Vec<ReleaseAdjustment> {
    let mut out: Vec<ReleaseAdjustment> = Vec::new();
    for item in items.iter() {
        let derived = new_release_adjustment(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn score_release_adjustment(item: &ReleaseAdjustment) -> String {
    canary_canary_forecast::canary_manifest_bucket(item.projected_load())
}

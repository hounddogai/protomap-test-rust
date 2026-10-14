use super::*;

use std::collections::BTreeMap;

use crate::canary::checkpoint as canary_checkpoint;

pub fn summarize_canary_forecasts_for_baseline(items: &[canary_checkpoint::CanaryForecast]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_release_baselines_from_forecast(items: &[canary_checkpoint::CanaryForecast]) -> Vec<ReleaseBaseline> {
    let mut out: Vec<ReleaseBaseline> = Vec::new();
    for item in items.iter() {
        let derived = new_release_baseline(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn balance_release_baseline(item: &ReleaseBaseline) -> String {
    canary_checkpoint::classify_canary_checkpoint(item.weighted_total())
}
